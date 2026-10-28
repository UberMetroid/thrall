//! Pure-Rust RFC 4511 LDAPv3 BER client for service bind, user DN lookup, user authentication,
//! and directory-wide LDAP user enumeration.

use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::device_store::sanitize_username;
use crate::fleet_config::LdapConfig;

fn encode_ber_length(len: usize) -> Vec<u8> {
    if len < 128 {
        vec![len as u8]
    } else if len <= 0xFF {
        vec![0x81, len as u8]
    } else {
        vec![0x82, ((len >> 8) & 0xFF) as u8, (len & 0xFF) as u8]
    }
}

fn ber_tlv(tag: u8, payload: &[u8]) -> Vec<u8> {
    let len_bytes = encode_ber_length(payload.len());
    let mut out = Vec::with_capacity(1 + len_bytes.len() + payload.len());
    out.push(tag);
    out.extend_from_slice(&len_bytes);
    out.extend_from_slice(payload);
    out
}

fn ber_int(val: i32) -> Vec<u8> {
    ber_tlv(0x02, &[val as u8])
}

fn ber_octet_str(s: &str) -> Vec<u8> {
    ber_tlv(0x04, s.as_bytes())
}

pub fn build_simple_bind_request(msg_id: i32, bind_dn: &str, password: &str) -> Vec<u8> {
    let mut bind_body = ber_int(3);
    bind_body.extend_from_slice(&ber_octet_str(bind_dn));
    bind_body.extend_from_slice(&ber_tlv(0x80, password.as_bytes()));
    let mut msg = ber_int(msg_id);
    msg.extend_from_slice(&ber_tlv(0x60, &bind_body));
    ber_tlv(0x30, &msg)
}

fn encode_equality_filter(attr: &str, val: &str) -> Vec<u8> {
    let mut eq = ber_octet_str(attr);
    eq.extend_from_slice(&ber_octet_str(val));
    ber_tlv(0xA3, &eq)
}

pub fn encode_ldap_filter(filter_str: &str, username: &str) -> Vec<u8> {
    let clean: String = username
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .collect();
    let rendered = filter_str
        .replace("%s", &clean)
        .replace("{{ login }}", &clean);
    let trimmed = rendered.trim();
    if let Some(inner) = trimmed.strip_prefix("(&").and_then(|s| s.strip_suffix(')')) {
        let mut children = Vec::new();
        let mut rest = inner.trim();
        while let Some(start) = rest.find('(') {
            let Some(end) = rest[start..].find(')') else {
                break;
            };
            if let Some((k, v)) = rest[start + 1..start + end].split_once('=') {
                children.extend_from_slice(&encode_equality_filter(k.trim(), v.trim()));
            }
            rest = &rest[start + end + 1..];
        }
        if !children.is_empty() {
            return ber_tlv(0xA0, &children);
        }
    }
    let inner = trimmed
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(trimmed);
    if let Some((k, v)) = inner.split_once('=') {
        return encode_equality_filter(k.trim(), v.trim());
    }
    encode_equality_filter("uid", &clean)
}

pub fn build_search_request(msg_id: i32, base_dn: &str, filter_ber: &[u8]) -> Vec<u8> {
    let mut body = ber_octet_str(base_dn);
    body.extend_from_slice(&[0x0A, 1, 2, 0x0A, 1, 0, 0x02, 1, 100, 0x02, 1, 5, 0x01, 1, 0]);
    body.extend_from_slice(filter_ber);
    body.extend_from_slice(&ber_tlv(0x30, &ber_octet_str("uid")));
    let mut msg = ber_int(msg_id);
    msg.extend_from_slice(&ber_tlv(0x63, &body));
    ber_tlv(0x30, &msg)
}

fn read_ldap_packet(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut hdr = [0u8; 2];
    stream.read_exact(&mut hdr).map_err(|e| e.to_string())?;
    if hdr[0] != 0x30 {
        return Err("Invalid LDAP SEQUENCE tag".to_string());
    }
    let len = if hdr[1] & 0x80 == 0 {
        usize::from(hdr[1])
    } else {
        let n = usize::from(hdr[1] & 0x7F);
        if n == 0 || n > 3 {
            return Err("Unsupported BER length".to_string());
        }
        let mut lbuf = vec![0u8; n];
        stream.read_exact(&mut lbuf).map_err(|e| e.to_string())?;
        lbuf.into_iter()
            .fold(0usize, |acc, b| (acc << 8) | usize::from(b))
    };
    if len > 65_536 {
        return Err("LDAP packet exceeds 64 KiB".to_string());
    }
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).map_err(|e| e.to_string())?;
    Ok(payload)
}

fn parse_tlv_slice(buf: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let (&tag, &len0) = (buf.first()?, buf.get(1)?);
    let (hdr_len, len) = if len0 & 0x80 == 0 {
        (2usize, usize::from(len0))
    } else {
        let c = usize::from(len0 & 0x7F);
        if c == 0 || c > 3 || buf.len() < 2 + c {
            return None;
        }
        let l = buf[2..2 + c]
            .iter()
            .fold(0usize, |acc, &b| (acc << 8) | usize::from(b));
        (2 + c, l)
    };
    (buf.len() >= hdr_len + len).then(|| (tag, &buf[hdr_len..hdr_len + len], &buf[hdr_len + len..]))
}

pub fn parse_ldap_op(packet_body: &[u8]) -> Option<(u8, &[u8])> {
    let (_, _, rest) = parse_tlv_slice(packet_body)?;
    let (op_tag, op_body, _) = parse_tlv_slice(rest)?;
    Some((op_tag, op_body))
}

fn send_bind(stream: &mut TcpStream, msg_id: i32, dn: &str, pass: &str) -> Result<(), String> {
    let req = build_simple_bind_request(msg_id, dn, pass);
    stream.write_all(&req).map_err(|e| e.to_string())?;
    let resp = read_ldap_packet(stream)?;
    let Some((0x61, body)) = parse_ldap_op(&resp) else {
        return Err("Unexpected auth response".to_string());
    };
    if parse_tlv_slice(body).and_then(|(_, c, _)| c.first().copied()) != Some(0) {
        return Err("Invalid user or password".to_string());
    }
    Ok(())
}

fn connect_and_service_bind(cfg: &LdapConfig) -> Result<TcpStream, String> {
    let u = cfg.url.trim().trim_end_matches('/');
    let hp = u
        .strip_prefix("ldap://")
        .or_else(|| u.strip_prefix("ldaps://"))
        .unwrap_or(u);
    let addr = hp
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("Directory unreachable")?;
    let to = Duration::from_secs(5);
    let mut stream = TcpStream::connect_timeout(&addr, to).map_err(|e| e.to_string())?;
    let _ = (
        stream.set_read_timeout(Some(to)),
        stream.set_write_timeout(Some(to)),
    );
    send_bind(&mut stream, 1, &cfg.bind_dn, &cfg.bind_password)?;
    Ok(stream)
}

fn collect_search_dns(stream: &mut TcpStream, base_dn: &str, filter: &[u8]) -> Vec<String> {
    let mut dns = Vec::new();
    let req = build_search_request(2, base_dn, filter);
    if stream.write_all(&req).is_ok() {
        for _ in 0..128 {
            let Ok(pkt) = read_ldap_packet(stream) else {
                break;
            };
            let Some((op_tag, op_body)) = parse_ldap_op(&pkt) else {
                break;
            };
            if op_tag == 0x64 {
                if let Some((0x04, dn_bytes, _)) = parse_tlv_slice(op_body) {
                    dns.push(String::from_utf8_lossy(dn_bytes).to_string());
                }
            } else if op_tag == 0x65 {
                break;
            }
        }
    }
    dns
}

#[rustfmt::skip]
pub fn list_ldap_users(cfg: &LdapConfig) -> Vec<String> {
    let mut users: BTreeSet<String> = cfg.admin_users.iter().filter_map(|a| sanitize_username(a).ok()).filter(|u| u != "root").collect();
    if let Ok(mut stream) = connect_and_service_bind(cfg) {
        let filter = encode_equality_filter("objectClass", "person");
        for dn in collect_search_dns(&mut stream, &cfg.base_dn, &filter) {
            if let Some(uid) = dn.split(',').next().unwrap_or("").trim().strip_prefix("uid=") {
                if let Ok(clean) = sanitize_username(uid) {
                    if clean != "root" { users.insert(clean); }
                }
            }
        }
    }
    users.into_iter().collect()
}

pub fn validate_ldap_connection(cfg: &LdapConfig) -> Result<usize, String> {
    let mut stream = connect_and_service_bind(cfg)?;
    let filter = encode_equality_filter("objectClass", "person");
    Ok(collect_search_dns(&mut stream, &cfg.base_dn, &filter).len())
}

#[rustfmt::skip]
pub fn authenticate_ldap_user(cfg: &LdapConfig, u: &str, p: &str) -> Result<String, String> {
    if u.trim().is_empty() || p.is_empty() { return Err("User and password are required".to_string()); }
    let mut stream = connect_and_service_bind(cfg)?;
    let filter = encode_ldap_filter(&cfg.user_filter, u);
    let user_dn = collect_search_dns(&mut stream, &cfg.base_dn, &filter).into_iter().next().ok_or("Invalid credentials")?;
    send_bind(&mut stream, 3, &user_dn, p)?;
    Ok(user_dn)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    #[test]
    fn test_ber_encoding_for_bind_and_search() {
        assert_eq!(build_simple_bind_request(1, "cn=a", "s")[0], 0x30);
        let f = encode_ldap_filter("(&(objectClass=person)(uid=%s))", "alice");
        assert_eq!((f[0], build_search_request(2, "dc=a", &f)[0]), (0xA0, 0x30));
    }
}
