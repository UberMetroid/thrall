//! Path-jailed Web File Explorer resolution, public share tokens, and URL form decoding for THRALL.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::device_store::{sanitize_folder_name, sanitize_username, MeshStore};

pub fn url_decode(input: &str) -> String {
    let b = input.as_bytes();
    let (mut out, mut i) = (Vec::with_capacity(b.len()), 0);
    while i < b.len() {
        if b[i] == b'+' {
            out.push(b' ');
            i += 1;
        } else if b[i] == b'%' && i + 2 < b.len() {
            let h = (b[i + 1] as char).to_digit(16);
            let l = (b[i + 2] as char).to_digit(16);
            if let (Some(hv), Some(lv)) = (h, l) {
                out.push(((hv << 4) | lv) as u8);
                i += 3;
                continue;
            }
            out.push(b[i]);
            i += 1;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

pub fn decode_hex(hex: &str) -> Vec<u8> {
    let b = hex.trim().as_bytes();
    let (mut out, mut i) = (Vec::with_capacity(b.len() / 2), 0);
    while i + 1 < b.len() {
        if let (Some(h), Some(l)) = ((b[i] as char).to_digit(16), (b[i + 1] as char).to_digit(16)) {
            out.push(((h << 4) | l) as u8);
        }
        i += 2;
    }
    out
}

pub fn parse_form_urlencoded(body: &str) -> BTreeMap<String, String> {
    body.split('&')
        .filter_map(|p| p.split_once('='))
        .map(|(k, v)| (url_decode(k.trim()), url_decode(v)))
        .collect()
}

pub fn parse_form_multi(body: &str, key: &str) -> BTreeSet<String> {
    body.split('&')
        .filter_map(|p| p.split_once('='))
        .filter(|(k, _)| url_decode(k.trim()) == key)
        .filter_map(|(_, v)| {
            let d = url_decode(v);
            if key == "users" {
                sanitize_username(&d).ok()
            } else {
                sanitize_folder_name(&d).ok()
            }
        })
        .collect()
}

pub fn sanitize_rel_subpath(sub: &str) -> Result<String, String> {
    let mut clean = Vec::new();
    for seg in sub.trim().trim_matches('/').split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." || seg.starts_with('.') || seg.contains('\0') || seg.contains('\\') {
            return Err("Unsafe subpath".to_string());
        }
        clean.push(seg);
    }
    Ok(clean.join("/"))
}

pub fn verify_no_symlink_escape(base: &std::path::Path, full: &std::path::Path) -> bool {
    let mut cur = Some(full);
    while let Some(p) = cur {
        if let Ok(sm) = std::fs::symlink_metadata(p) {
            if sm.file_type().is_symlink() {
                return false;
            }
            return match (base.canonicalize(), p.canonicalize()) {
                (Ok(cb), Ok(cp)) => cp.starts_with(&cb),
                _ => false,
            };
        }
        if p == base {
            break;
        }
        cur = p.parent();
    }
    true
}

pub fn resolve_jailed_path(
    user: &str,
    store: &MeshStore,
    browse: &str,
    sub: &str,
) -> Option<(String, PathBuf, String)> {
    let rel = sanitize_rel_subpath(sub).ok()?;
    let ufolders = store.folders_for_user(user);
    let (label, base) = if browse.is_empty() || browse == "personal" {
        let f = ufolders.first()?.clone();
        (
            format!("Personal / {f}"),
            PathBuf::from(format!("/users/{user}/{f}")),
        )
    } else if let Some(pf) = browse
        .strip_prefix("personal:")
        .or_else(|| browse.strip_prefix("personal/"))
    {
        if !ufolders.iter().any(|x| x == pf) {
            return None;
        }
        (
            format!("Personal / {pf}"),
            PathBuf::from(format!("/users/{user}/{pf}")),
        )
    } else {
        let sh = browse.strip_prefix("shared:")?;
        if !store.shared_folders.get(sh)?.contains(user) {
            return None;
        }
        let p = if sh == "Shared" {
            "/shared".into()
        } else {
            format!("/shared/{sh}")
        };
        (format!("Shared ({sh})"), PathBuf::from(p))
    };
    let full = if rel.is_empty() {
        base.clone()
    } else {
        base.join(&rel)
    };
    verify_no_symlink_escape(&base, &full).then_some((label, full, rel))
}

pub fn compute_share_token(target: &str) -> String {
    let mut h = 0xcbf29ce484222325u64;
    for &b in target.as_bytes() {
        h = (h ^ u64::from(b)).wrapping_mul(0x100000001b3);
    }
    format!("{:010x}", h & 0x0000_00ff_ffff_ffff)
}

pub fn syncthing_folder_id(seed: &str) -> String {
    const ALPHA: &[u8; 32] = b"abcdefghijkmnpqrstuvwxyz23456790";
    let h = prf_mac64(b"THRALL_ST_FOLDER_ID", seed.as_bytes(), 0x53545f4649445f31);
    let mut out = String::with_capacity(11);
    for i in 0..10 {
        if i == 5 {
            out.push('-');
        }
        out.push(ALPHA[((h >> (i * 5)) & 31) as usize] as char);
    }
    out
}

pub fn resolve_share_target(target: &str, sub: &str) -> Option<PathBuf> {
    let (scope, rel_t) = target.split_once('/').unwrap_or((target, ""));
    let base = if let Some(u) = scope.strip_prefix("user:") {
        PathBuf::from(format!("/users/{}", sanitize_username(u).ok()?))
    } else if let Some(rest) = scope.strip_prefix("personal:") {
        let (u, pf) = rest.split_once(':')?;
        let u_clean = sanitize_username(u).ok()?;
        let pf_clean = sanitize_folder_name(pf).ok()?;
        PathBuf::from(format!("/users/{u_clean}/{pf_clean}"))
    } else {
        let sh = scope.strip_prefix("shared:")?;
        let s = sanitize_folder_name(sh).ok()?;
        PathBuf::from(if s == "Shared" {
            "/shared".into()
        } else {
            format!("/shared/{s}")
        })
    };
    let p1 = sanitize_rel_subpath(rel_t).ok()?;
    let p2 = sanitize_rel_subpath(sub).ok()?;
    let mid = if p1.is_empty() {
        base.clone()
    } else {
        base.join(p1)
    };
    let full = if p2.is_empty() { mid } else { mid.join(p2) };
    verify_no_symlink_escape(&base, &full).then_some(full)
}

fn prf_mac64(key: &[u8], data: &[u8], domain: u64) -> u64 {
    let mut s0 = 0x736f6d6570736575u64 ^ domain;
    let mut s1 = 0x646f72616e646f6du64 ^ (key.len() as u64);
    for (i, &b) in key.iter().chain(data.iter()).enumerate() {
        s0 = s0.wrapping_add(u64::from(b).wrapping_mul(0x9e3779b97f4a7c15));
        s1 = (s1 ^ s0).rotate_left(29).wrapping_mul(0xbf58476d1ce4e5b9);
        s0 = (s0 ^ (i as u64)).rotate_left(17).wrapping_add(s1);
    }
    for _ in 0..4 {
        s0 = s0.wrapping_add(s1);
        s1 = s1.rotate_left(13) ^ s0;
        s0 = s0.rotate_left(32);
    }
    s0 ^ s1
}

pub fn sign_session_token(username: &str, exp_epoch: u64, secret: &str) -> String {
    let payload = format!("{username}:{exp_epoch}");
    let h1 = prf_mac64(secret.as_bytes(), payload.as_bytes(), 0x544852414c4c5f31);
    let h2 = prf_mac64(secret.as_bytes(), payload.as_bytes(), 0x544852414c4c5f32);
    format!("{payload}:{h1:016x}{h2:016x}")
}

pub fn verify_session_token(token: &str, secret: &str, now_epoch: u64) -> Option<String> {
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let username = sanitize_username(parts[0]).ok()?;
    let exp: u64 = parts[1].parse().ok()?;
    if exp < now_epoch {
        return None;
    }
    let expected = sign_session_token(&username, exp, secret);
    let diff = expected
        .bytes()
        .zip(token.bytes())
        .fold(0u8, |a, (x, y)| a | (x ^ y));
    (expected.len() == token.len() && diff == 0).then_some(username)
}

pub fn extract_cookie(headers: &str, cookie_name: &str) -> Option<String> {
    headers
        .lines()
        .filter(|l| l.to_ascii_lowercase().starts_with("cookie:"))
        .flat_map(|l| l[7..].split(';'))
        .filter_map(|item| item.trim().split_once('='))
        .find(|(k, _)| k.trim() == cookie_name)
        .map(|(_, v)| v.trim().to_string())
}
