//! HTTP/1.1 portal router, keyed session cookie authenticator, and form handler.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::device_store::{
    resolve_jailed_path, resolve_share_target, sanitize_username, MeshStore,
};
use crate::fleet_config::FleetConfig;
use crate::ldap_bind::authenticate_ldap_user;
use crate::reconcile_mesh::FleetSnapshot;
use crate::render_portal::explorer_view::render_public_share_dir;
use crate::render_portal::portal_actions;
pub use crate::render_portal::share_jail::{
    extract_cookie, parse_form_multi, parse_form_urlencoded, sign_session_token, url_decode,
    verify_session_token,
};
use crate::render_portal::{render_dashboard_page, render_login_page, render_telemetry_json};

#[rustfmt::skip]
const THRALL_BGS: [&[u8]; 5] = [include_bytes!("../../assets/thrall-bg-1.jpg"), include_bytes!("../../assets/thrall-bg-2.jpg"), include_bytes!("../../assets/thrall-bg-3.jpg"), include_bytes!("../../assets/thrall-bg-4.jpg"), include_bytes!("../../assets/thrall-bg.jpg")];
const FAVICON_SVG: &[u8] = include_bytes!("../../assets/icons/favicon.svg");
const FAVICON_ICO: &[u8] = include_bytes!("../../assets/icons/favicon.ico");
const CEL_ICON_JPG: &[u8] = include_bytes!("../../assets/icons/thrall-cel-icon.jpg");
const MASCOT_JPG: &[u8] = include_bytes!("../../assets/thrall-mascot.jpg");
const LOGIN_JS: &[u8] = include_bytes!("../../assets/login.js");
const SEC_HDRS: &str = "X-Frame-Options: DENY\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nCross-Origin-Opener-Policy: same-origin\r\nContent-Security-Policy: default-src 'none'; script-src 'self'; connect-src 'self'; style-src 'unsafe-inline'; img-src 'self' data:; form-action 'self'; frame-ancestors 'none'; base-uri 'none'\r\nConnection: close\r\n";

pub(crate) fn write_http(s: &mut TcpStream, status: &str, ct: &str, extra: &str, body: &[u8]) {
    let len = body.len();
    let hdr = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ct}\r\nContent-Length: {len}\r\n{SEC_HDRS}{extra}\r\n"
    );
    let _ = (s.write_all(hdr.as_bytes()), s.write_all(body));
}

pub(crate) fn write_html(s: &mut TcpStream, status: &str, body: &str) {
    let ct = "text/html; charset=utf-8";
    write_http(s, status, ct, "", body.as_bytes());
}

pub(crate) fn write_redirect(s: &mut TcpStream, loc: &str, ck: &str) {
    write_http(
        s,
        "303 See Other",
        "text/plain",
        &format!("Location: {loc}\r\n{ck}"),
        b"",
    );
}

fn serve_file(stream: &mut TcpStream, fpath: &std::path::Path) -> bool {
    let Ok(m) = fs::symlink_metadata(fpath) else {
        return false;
    };
    if !m.is_file() || m.len() > 67_108_864 {
        return false;
    }
    if let Ok(mut file) = fs::File::open(fpath) {
        let fnm = fpath.file_name().and_then(|s| s.to_str()).unwrap_or("file");
        let len = m.len();
        let hdr = format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {len}\r\n{SEC_HDRS}Content-Disposition: inline; filename=\"{fnm}\"\r\n\r\n");
        let _ = (
            stream.write_all(hdr.as_bytes()),
            std::io::copy(&mut file, stream),
        );
        return true;
    }
    false
}

fn read_http_req(stream: &mut TcpStream, buf: &mut [u8]) -> usize {
    let mut n = stream.read(buf).unwrap_or(0);
    while n > 0 && n < buf.len() {
        let s = String::from_utf8_lossy(&buf[..n]);
        let Some((h, b)) = s.split_once("\r\n\r\n") else {
            break;
        };
        let clen = h
            .lines()
            .find(|l| l.to_ascii_lowercase().starts_with("content-length:"))
            .and_then(|l| l[15..].trim().parse::<usize>().ok())
            .unwrap_or(0);
        if b.len() >= clen {
            break;
        }
        let m = stream.read(&mut buf[n..]).unwrap_or(0);
        if m == 0 {
            break;
        }
        n += m;
    }
    n
}

pub fn handle_http_client(
    mut stream: TcpStream,
    cfg: &mut FleetConfig,
    store: &mut MeshStore,
    snap: &mut FleetSnapshot,
) {
    let to = Some(Duration::from_secs(5));
    let _ = (stream.set_read_timeout(to), stream.set_write_timeout(to));
    let mut buf = [0u8; 65_536];
    let n = read_http_req(&mut stream, &mut buf);
    if n == 0 {
        return;
    }
    let raw_req = String::from_utf8_lossy(&buf[..n]);
    let (headers, body) = raw_req.split_once("\r\n\r\n").unwrap_or((&raw_req, ""));
    let mut parts = headers.lines().next().unwrap_or("").split_whitespace();
    let (method, raw_path) = (parts.next().unwrap_or("GET"), parts.next().unwrap_or("/"));
    let (path, query) = raw_path.split_once('?').unwrap_or((raw_path, ""));
    let qmap = parse_form_urlencoded(query);
    let qval = |k: &str| qmap.get(k).map(String::as_str).unwrap_or("");
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let now = dur.as_secs();

    let s = &mut stream;
    match path {
        "/favicon.svg" => return write_http(s, "200 OK", "image/svg+xml", "", FAVICON_SVG),
        "/favicon.ico" => return write_http(s, "200 OK", "image/x-icon", "", FAVICON_ICO),
        "/icon.jpg" => return write_http(s, "200 OK", "image/jpeg", "", CEL_ICON_JPG),
        "/mascot.jpg" => return write_http(s, "200 OK", "image/jpeg", "", MASCOT_JPG),
        "/login.js" => {
            return write_http(
                s,
                "200 OK",
                "application/javascript; charset=utf-8",
                "",
                LOGIN_JS,
            )
        }
        "/healthz" | "/status" => {
            let on = snap.nodes.iter().filter(|n| n.online).count();
            let (tot, ulen) = (snap.nodes.len(), store.users.len());
            let j = format!("{{\"status\":\"healthy\",\"online_nodes\":{on},\"total_nodes\":{tot},\"registered_users\":{ulen}}}\n");
            return write_http(s, "200 OK", "application/json", "", j.as_bytes());
        }
        "/api/telemetry" => {
            let j = render_telemetry_json(snap, store);
            return write_http(
                s,
                "200 OK",
                "application/json",
                "Cache-Control: no-store\r\n",
                j.as_bytes(),
            );
        }
        _ if path.starts_with("/bg") && path.ends_with(".jpg") => {
            let idx = match path {
                "/bg-1.jpg" => 0,
                "/bg-2.jpg" => 1,
                "/bg-3.jpg" => 2,
                "/bg-4.jpg" => 3,
                "/bg-5.jpg" => 4,
                _ => ((dur.subsec_nanos() as usize).wrapping_add(now as usize)) % THRALL_BGS.len(),
            };
            let cc = "Cache-Control: public, max-age=3600\r\n";
            return write_http(s, "200 OK", "image/jpeg", cc, THRALL_BGS[idx]);
        }
        _ => {}
    }
    if let Some(rest) = path.strip_prefix("/s/") {
        let (tok, psub) = rest.split_once('/').unwrap_or((rest, ""));
        let esub = if qval("sub").is_empty() {
            psub
        } else {
            qval("sub")
        };
        if let Some(p) = store
            .public_shares
            .get(tok)
            .and_then(|t| resolve_share_target(t, esub))
        {
            if p.is_dir() {
                return write_html(s, "200 OK", &render_public_share_dir(tok, esub, &p));
            } else if serve_file(s, &p) {
                return;
            }
        }
        return write_http(s, "404 Not Found", "text/plain", "", b"Not found");
    }
    let secret = &cfg.portal.session_secret;
    let session_user = extract_cookie(headers, "thrall_session")
        .and_then(|t| verify_session_token(&t, secret, now));
    let form = parse_form_urlencoded(body);
    let fval = |k: &str| form.get(k).map(String::as_str).unwrap_or("");

    if method == "GET" && path == "/login" {
        return write_html(s, "200 OK", &render_login_page(None, snap, store));
    }
    if method == "POST" && path == "/login" {
        let Ok(user) = sanitize_username(fval("username")) else {
            let p = render_login_page(Some("Invalid user syntax"), snap, store);
            return write_html(s, "400 Bad Request", &p);
        };
        return match authenticate_ldap_user(&cfg.ldap, &user, fval("password")) {
            Ok(_) => {
                let tok = sign_session_token(&user, now + 43_200, secret);
                let ck = format!("Set-Cookie: thrall_session={tok}; Path=/; HttpOnly; SameSite=Lax; Max-Age=43200\r\n");
                write_redirect(s, "/", &ck)
            }
            Err(e) => write_html(
                s,
                "401 Unauthorized",
                &render_login_page(Some(&e), snap, store),
            ),
        };
    }
    if method == "POST" && path == "/logout" {
        let ck = "Set-Cookie: thrall_session=deleted; Path=/; HttpOnly; Max-Age=0\r\n";
        return write_redirect(s, "/login", ck);
    }

    let Some(u) = session_user else {
        return write_html(s, "200 OK", &render_login_page(None, snap, store));
    };
    let adm = cfg.ldap.admin_users.iter().any(|x| x == &u);

    if method == "GET" && path == "/files" {
        if let Some((_, fp, _)) = resolve_jailed_path(&u, store, qval("browse"), qval("sub")) {
            if serve_file(s, &fp) {
                return;
            }
        }
        return write_http(s, "404 Not Found", "text/plain", "", b"Not found");
    }
    if method == "POST"
        && portal_actions::handle_post_action(s, path, body, &form, &u, adm, cfg, store, snap)
    {
        return;
    }
    let tab: u8 = match qval("tab") {
        "shared" => 2,
        "devices" => 3,
        "admin" | "global" | "mesh" | "ldap" => 4,
        "about" => 6,
        _ => 1,
    };
    let (ok, err) = (
        qmap.get("ok").map(String::as_str),
        qmap.get("err").map(String::as_str),
    );
    let (br, sb) = (qval("browse"), qval("sub"));
    let page = render_dashboard_page(&u, adm, cfg, store, snap, tab, br, sb, ok, err);
    write_html(s, "200 OK", &page);
}
