//! Syncthing 56-character Device ID normalization and Base32 Luhn mod 32 check-digit verifier,
//! plus `/config/mesh-mgr.db` persistence and path-jailed web file explorer resolution.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub use crate::render_portal::share_jail::{
    compute_share_token, resolve_jailed_path, resolve_share_target, sanitize_rel_subpath,
    syncthing_folder_id,
};

const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
#[rustfmt::skip]
pub const XDG_FOLDERS: [&str; 8] = ["Desktop", "Documents", "Downloads", "Music", "Pictures", "Videos", "Public", "Templates"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserDevice {
    pub device_id: String,
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MeshStore {
    pub users: BTreeMap<String, Vec<UserDevice>>,
    pub user_folders: BTreeMap<String, Vec<String>>,
    pub shared_folders: BTreeMap<String, BTreeSet<String>>,
    pub disabled_servers: BTreeSet<String>,
    pub public_shares: BTreeMap<String, String>,
    pub folder_paths: BTreeMap<(String, String), String>,
}

fn luhn_base32_check_char(chunk13: &[u8]) -> Option<u8> {
    if chunk13.len() != 13 {
        return None;
    }
    let (mut factor, mut sum) = (1u32, 0u32);
    for &b in chunk13 {
        let v = BASE32_ALPHABET.iter().position(|&c| c == b)? as u32;
        let addend = factor * v;
        factor = if factor == 2 { 1 } else { 2 };
        sum += (addend / 32) + (addend % 32);
    }
    BASE32_ALPHABET
        .get(((32 - (sum % 32)) % 32) as usize)
        .copied()
}

pub fn normalize_and_verify_device_id(raw: &str) -> Result<String, String> {
    let cleaned: Vec<u8> = raw
        .bytes()
        .map(|b| b.to_ascii_uppercase())
        .filter(|b| b.is_ascii_alphanumeric())
        .collect();
    if cleaned.len() < 56 {
        return Err("Device ID must contain 56 Base32 chars".to_string());
    }
    for win in cleaned.windows(56) {
        let valid = (0..4).all(|g| {
            let grp = &win[g * 14..(g + 1) * 14];
            luhn_base32_check_char(&grp[0..13]) == Some(grp[13])
        });
        if valid {
            let grouped: Vec<String> = win
                .chunks(7)
                .map(|c| String::from_utf8_lossy(c).into())
                .collect();
            return Ok(grouped.join("-"));
        }
    }
    Err("Invalid Syncthing Device ID or Luhn check digit".to_string())
}

pub fn sanitize_username(username: &str) -> Result<String, String> {
    let raw = username.trim().split('@').next().unwrap_or("");
    let t = raw.trim().to_ascii_lowercase();
    let ok = !t.is_empty()
        && t.len() <= 64
        && t.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    ok.then_some(t)
        .ok_or_else(|| "Invalid username".to_string())
}

pub fn sanitize_folder_name(name: &str) -> Result<String, String> {
    let t = name.trim().replace(' ', "-");
    let ok = !t.is_empty()
        && t.len() <= 64
        && !t.starts_with('.')
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    ok.then_some(t)
        .ok_or_else(|| "Invalid folder name".to_string())
}

pub fn sanitize_device_label(label: &str) -> Result<String, String> {
    let cleaned: String = label
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ' '))
        .take(48)
        .collect();
    let t = cleaned.trim();
    (!t.is_empty())
        .then(|| t.replace(' ', "-"))
        .ok_or_else(|| "Empty label".to_string())
}

#[rustfmt::skip]
pub fn sanitize_local_path(raw: &str) -> Result<String, String> {
    let t = raw.trim().replace('\\', "/");
    let ok = !t.is_empty() && t.len() <= 128 && !t.contains("..") && t.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '~' | '/' | ':' | '.' | '-' | '_' | ' '));
    ok.then_some(t).ok_or_else(|| "Invalid local path".to_string())
}

#[rustfmt::skip]
pub fn default_local_path(scope_folder: &str) -> String {
    let name = scope_folder.split_once(':').map_or(scope_folder, |(_, f)| f);
    if scope_folder.starts_with("personal:") && XDG_FOLDERS.iter().any(|x| x.eq_ignore_ascii_case(name)) {
        format!("~/{name}")
    } else {
        format!("~/Thrall/{name}")
    }
}

#[rustfmt::skip]
impl MeshStore {
    pub fn load(path: &str) -> Self {
        fs::read_to_string(path).map(|c| Self::parse(&c)).unwrap_or_default()
    }

    pub fn folders_for_user(&self, user: &str) -> Vec<String> {
        self.user_folders.get(user).cloned().unwrap_or_default()
    }

    pub fn ensure_user_folders(&mut self, user: &str) -> &mut Vec<String> {
        self.user_folders.entry(user.to_string()).or_default()
    }

    pub fn local_path_for(&self, user: &str, scope_folder: &str) -> String {
        self.folder_paths.get(&(user.to_string(), scope_folder.to_string())).cloned().unwrap_or_else(|| default_local_path(scope_folder))
    }

    pub fn parse(content: &str) -> Self {
        let mut store = Self::default();
        for l in content.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            if let Some((f, csv)) = l.strip_prefix("@share\t").and_then(|r| r.split_once('\t')) {
                if let Ok(fname) = sanitize_folder_name(f) {
                    let set = store.shared_folders.entry(fname).or_default();
                    for u in csv.split(',').filter_map(|x| sanitize_username(x).ok()) { set.insert(u); }
                }
            } else if let Some(rest) = l.strip_prefix("@user_folders\t") {
                let (u, csv) = rest.split_once('\t').unwrap_or((rest, ""));
                if let Ok(uname) = sanitize_username(u) {
                    if uname != "root" {
                        let f = csv.split(',').filter_map(|x| sanitize_folder_name(x).ok()).collect();
                        store.user_folders.insert(uname, f);
                    }
                }
            } else if let Some(rest) = l.strip_prefix("@folder_path\t") {
                let p: Vec<&str> = rest.splitn(3, '\t').collect();
                if let (Some(&u), Some(&sf), Some(&lp)) = (p.first(), p.get(1), p.get(2)) {
                    let ok_sf = sf.strip_prefix("personal:").or_else(|| sf.strip_prefix("shared:")).is_some_and(|f| sanitize_folder_name(f).is_ok());
                    if let (true, Ok(un), Ok(cp)) = (ok_sf, sanitize_username(u), sanitize_local_path(lp)) {
                        store.folder_paths.insert((un, sf.to_string()), cp);
                    }
                }
            } else if let Some(srv) = l.strip_prefix("@disabled_server\t") {
                if !srv.trim().is_empty() { store.disabled_servers.insert(srv.trim().to_string()); }
            } else if let Some((tok, tgt)) = l.strip_prefix("@link\t").and_then(|r| r.split_once('\t')) {
                if !tok.is_empty() && !tgt.is_empty() { store.public_shares.insert(tok.to_string(), tgt.to_string()); }
            } else {
                let p: Vec<&str> = l.splitn(3, '\t').collect();
                if p.len() >= 2 { let _ = store.upsert_device(p[0], p[1], p.get(2).copied().unwrap_or("")); }
            }
        }
        store
    }

    pub fn serialize(&self) -> String {
        let mut out = String::from("# THRALL mesh-mgr.db v2\n");
        for (user, devices) in &self.users {
            for dev in devices { out.push_str(&format!("{user}\t{}\t{}\n", dev.device_id, dev.label)); }
        }
        for (user, folders) in &self.user_folders {
            out.push_str(&format!("@user_folders\t{user}\t{}\n", folders.join(",")));
        }
        for ((user, sf), lp) in &self.folder_paths {
            out.push_str(&format!("@folder_path\t{user}\t{sf}\t{lp}\n"));
        }
        for (folder, users) in &self.shared_folders {
            let joined: Vec<&str> = users.iter().map(String::as_str).collect();
            out.push_str(&format!("@share\t{folder}\t{}\n", joined.join(",")));
        }
        for srv in &self.disabled_servers { out.push_str(&format!("@disabled_server\t{srv}\n")); }
        for (tok, tgt) in &self.public_shares { out.push_str(&format!("@link\t{tok}\t{tgt}\n")); }
        out
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        if let Some(parent) = Path::new(path).parent() { let _ = fs::create_dir_all(parent); }
        let tmp = format!("{path}.tmp");
        fs::write(&tmp, self.serialize()).map_err(|e| e.to_string())?;
        fs::rename(&tmp, path).map_err(|e| e.to_string())
    }

    pub fn upsert_device(&mut self, user: &str, dev_id: &str, label: &str) -> Result<(), String> {
        let u = sanitize_username(user)?;
        let id = normalize_and_verify_device_id(dev_id)?;
        let _ = self.ensure_user_folders(&u);
        let list = self.users.entry(u).or_default();
        let idx = list.iter().filter_map(|d| d.label.split('-').nth(1)?.parse::<usize>().ok()).max().unwrap_or(list.len()) + 1;
        let short = id.split('-').next().unwrap_or("dev").to_ascii_lowercase();
        let lbl = sanitize_device_label(label).unwrap_or_else(|_| format!("dev-{idx}-{short}"));
        if let Some(existing) = list.iter_mut().find(|d| d.device_id == id) {
            if !label.trim().is_empty() { existing.label = lbl; }
        } else {
            list.push(UserDevice { device_id: id, label: lbl });
        }
        Ok(())
    }

    pub fn remove_device(&mut self, username: &str, device_id: &str) -> Result<bool, String> {
        let user = sanitize_username(username)?;
        let norm_id = normalize_and_verify_device_id(device_id)?;
        let Some(list) = self.users.get_mut(&user) else { return Ok(false) };
        let before = list.len();
        list.retain(|d| d.device_id != norm_id);
        Ok(list.len() != before)
    }
}
