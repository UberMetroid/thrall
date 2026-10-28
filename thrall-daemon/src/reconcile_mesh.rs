//! Fleet Mesh Reconciliation (Server-to-Server) and Automated Per-User & Shared Folder Provisioning.

use std::collections::BTreeMap;
use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use crate::device_store::{sanitize_folder_name, MeshStore};
use crate::fleet_config::{FleetConfig, ServerNodeConfig};
use crate::ldap_bind::list_ldap_users;
use crate::peer_client::{
    discover_api_key, parse_json_string_field, parse_json_u64_field, syncthing_rest,
};

use crate::render_portal::reconcile_folders;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeStatus {
    pub name: String,
    pub api_url: String,
    pub wan_addr: String,
    pub online: bool,
    pub disabled: bool,
    pub device_id: String,
    pub api_key: String,
    pub uptime_sec: u64,
    pub active_conns: u64,
    pub in_bps: u64,
    pub out_bps: u64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub total_bytes: u64,
    pub rtt_ms: u64,
    pub bearing_deg: u64,
    pub connected_ids: Vec<String>,
    pub last_error: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FleetSnapshot {
    pub nodes: Vec<NodeStatus>,
    pub ldap_users: Vec<String>,
    pub user_completion: BTreeMap<(String, String), u64>,
    pub user_accepted: BTreeMap<String, bool>,
    pub shared_completion: BTreeMap<String, u64>,
    pub shared_user_accepted: BTreeMap<(String, String), bool>,
    pub device_accepted: BTreeMap<(String, String), bool>,
}

#[rustfmt::skip]
pub fn provision_single_folder(cfg: &FleetConfig, snap: &FleetSnapshot, store: &MeshStore, user: Option<&str>, fname: &str) {
    let online_ids: Vec<String> = snap.nodes.iter().filter(|n| n.online && !n.device_id.is_empty()).map(|n| n.device_id.clone()).collect();
    for (srv, st) in cfg.servers.iter().zip(snap.nodes.iter()).filter(|(_, s)| s.online) {
        let _ = reconcile_folders::sync_folder_group(srv, &st.api_key, &online_ids, store, user, fname, false);
    }
}

pub fn build_device_json(device_id: &str, name: &str, address: &str, introducer: bool) -> String {
    format!("{{\"deviceID\":\"{device_id}\",\"name\":\"{name}\",\"addresses\":[\"{address}\"],\"compression\":\"metadata\",\"introducer\":{introducer},\"autoAcceptFolders\":false}}")
}

#[rustfmt::skip]
pub fn build_folder_json(fid: &str, label: &str, path: &str, device_ids: &[String], ro: bool) -> String {
    let ftype = if ro { "sendonly" } else { "sendreceive" };
    let devs = device_ids.iter().map(|id| format!("{{\"deviceID\":\"{id}\",\"introducedBy\":\"\",\"encryptionPassword\":\"\"}}")).collect::<Vec<_>>().join(",");
    format!("{{\"id\":\"{fid}\",\"label\":\"{label}\",\"filesystemType\":\"basic\",\"path\":\"{path}\",\"type\":\"{ftype}\",\"devices\":[{devs}],\"rescanIntervalS\":3600,\"fsWatcherEnabled\":true,\"fsWatcherDelayS\":10,\"ignorePerms\":true,\"autoNormalize\":true}}")
}

#[rustfmt::skip]
fn tcp_ping_ms(wan_addr: &str) -> u64 {
    let raw = wan_addr.strip_prefix("tcp://").unwrap_or(wan_addr);
    let target = if raw.contains(':') { raw.to_string() } else { format!("{raw}:22000") };
    if let Some(addr) = target.to_ascii_lowercase().to_socket_addrs().ok().and_then(|mut i| i.next()) {
        let t0 = Instant::now();
        if TcpStream::connect_timeout(&addr, Duration::from_millis(800)).is_ok() {
            return (t0.elapsed().as_millis() as u64).max(1);
        }
    }
    0
}

#[rustfmt::skip]
pub fn parse_connected_device_ids(c1: &str) -> Vec<String> {
    let norm = c1.replace("\":{", "\": {");
    let (mut out, parts) = (Vec::new(), norm.split("\": {").collect::<Vec<_>>());
    for pair in parts.windows(2) {
        if let Some(id) = pair[0].rsplit_once('"').map(|(_, s)| s).filter(|s| s.len() >= 50) {
            let body = pair[1].split_once('}').map_or(pair[1], |(b, _)| b);
            if body.contains("\"connected\": true") || body.contains("\"connected\":true") { out.push(id.to_string()); }
        }
    }
    out
}

#[rustfmt::skip]
pub fn discover_node(node: &ServerNodeConfig, disabled: bool, prev: Option<&NodeStatus>) -> NodeStatus {
    let err = if disabled { "Disconnected by admin" } else { "" };
    let rtt = if disabled { 0 } else { tcp_ping_ms(&node.syncthing_wan_addr) };
    let mut h_hash = 0xcbf2_9ce4_8422_2325u64;
    for b in node.syncthing_api_url.bytes() { h_hash = (h_hash ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3); }
    let mut st = NodeStatus {
        name: node.name.clone(), api_url: node.syncthing_api_url.clone(), wan_addr: node.syncthing_wan_addr.clone(),
        online: false, disabled, device_id: String::new(), api_key: String::new(), uptime_sec: 0,
        active_conns: 0, in_bps: 0, out_bps: 0, rx_bytes: 0, tx_bytes: 0, total_bytes: 0, rtt_ms: rtt, bearing_deg: h_hash % 360,
        connected_ids: Vec::new(), last_error: err.to_string(),
    };
    if disabled { return st; }
    match discover_api_key(node) {
        Ok(k) => st.api_key = k,
        Err(e) => { st.last_error = e; return st; }
    }
    match syncthing_rest(node, &st.api_key, "GET", "/rest/system/status", None) {
        Ok(body) => {
            st.device_id = parse_json_string_field(&body, "myID").unwrap_or_default();
            st.uptime_sec = parse_json_u64_field(&body, "uptime").unwrap_or(0);
            st.online = !st.device_id.is_empty();
            if let Some(id_b) = st.device_id.bytes().next() { st.bearing_deg = (st.bearing_deg + u64::from(id_b) * 7) % 360; }
        }
        Err(e) => st.last_error = e,
    }
    if st.online {
        let totals = |b: &str| -> (u64, u64) {
            let t = b.rfind("\"total\"").map(|i| &b[i..]).unwrap_or(b);
            (parse_json_u64_field(t, "inBytesTotal").unwrap_or(0), parse_json_u64_field(t, "outBytesTotal").unwrap_or(0))
        };
        if let Ok(c1) = syncthing_rest(node, &st.api_key, "GET", "/rest/system/connections", None) {
            st.connected_ids = parse_connected_device_ids(&c1);
            st.active_conns = st.connected_ids.len() as u64;
            let (rx1, tx1) = totals(&c1);
            st.rx_bytes = rx1;
            st.tx_bytes = tx1;
            st.total_bytes = rx1 + tx1;
            if let Some(p) = prev.filter(|p| p.online && st.uptime_sec > p.uptime_sec) {
                let dt = (st.uptime_sec - p.uptime_sec).max(1);
                st.in_bps = (rx1.saturating_sub(p.rx_bytes) * 8) / dt;
                st.out_bps = (tx1.saturating_sub(p.tx_bytes) * 8) / dt;
            }
        }
    }
    st
}

pub fn verify_device_id_via_server(
    cfg: &FleetConfig,
    nodes: &[NodeStatus],
    dev_id: &str,
) -> Result<(), String> {
    let ep = format!("/rest/svc/deviceid?id={dev_id}");
    for (srv, st) in cfg
        .servers
        .iter()
        .zip(nodes.iter())
        .filter(|(_, s)| s.online)
    {
        if let Ok(resp) = syncthing_rest(srv, &st.api_key, "GET", &ep, None) {
            if let Some(err) = parse_json_string_field(&resp, "error") {
                return Err(format!("Syncthing rejected Device ID: {err}"));
            }
            if parse_json_string_field(&resp, "id").is_some() {
                return Ok(());
            }
        }
    }
    Ok(())
}

pub fn purge_device_from_fleet(cfg: &FleetConfig, nodes: &[NodeStatus], device_id: &str) {
    let path = format!("/rest/config/devices/{device_id}");
    for (srv, st) in cfg
        .servers
        .iter()
        .zip(nodes.iter())
        .filter(|(_, s)| s.online)
    {
        let _ = syncthing_rest(srv, &st.api_key, "DELETE", &path, None);
    }
}

pub fn purge_folder_from_fleet(cfg: &FleetConfig, nodes: &[NodeStatus], folder_id: &str) {
    let path = format!("/rest/config/folders/{folder_id}");
    for (srv, st) in cfg
        .servers
        .iter()
        .zip(nodes.iter())
        .filter(|(_, s)| s.online)
    {
        let _ = syncthing_rest(srv, &st.api_key, "DELETE", &path, None);
    }
}

fn discover_shared_folder_names(store: &mut MeshStore) -> Vec<String> {
    if let Ok(entries) = fs::read_dir("/shared") {
        for e in entries.flatten().filter(|e| e.path().is_dir()) {
            if let Some(n) = e.file_name().to_str().filter(|n| !n.starts_with('.')) {
                if let Ok(clean) = sanitize_folder_name(n) {
                    store.shared_folders.entry(clean).or_default();
                }
            }
        }
    }
    store.shared_folders.keys().cloned().collect()
}

#[rustfmt::skip]
pub fn reconcile_fleet(cfg: &FleetConfig, store: &mut MeshStore) -> FleetSnapshot {
    static PREV_NODES: std::sync::Mutex<Vec<NodeStatus>> = std::sync::Mutex::new(Vec::new());
    let prev_list = PREV_NODES.lock().ok().map(|g| g.clone()).unwrap_or_default();
    let mut snap = FleetSnapshot { ldap_users: list_ldap_users(&cfg.ldap), ..FleetSnapshot::default() };
    for u in store.users.keys() {
        if u != "root" && !snap.ldap_users.contains(u) { snap.ldap_users.push(u.clone()); }
    }
    snap.ldap_users.sort();
    for u in &snap.ldap_users { let _ = store.ensure_user_folders(u); }
    let folder_names = discover_shared_folder_names(store);
    for srv in &cfg.servers {
        let dis = store.disabled_servers.contains(&srv.name);
        let prev = prev_list.iter().find(|n| n.name == srv.name);
        snap.nodes.push(discover_node(srv, dis, prev));
    }
    if let Ok(mut g) = PREV_NODES.lock() { *g = snap.nodes.clone(); }
    let online_ids: Vec<String> = snap.nodes.iter().filter(|n| n.online && !n.device_id.is_empty()).map(|n| n.device_id.clone()).collect();
    let active_srvs: Vec<_> = cfg.servers.iter().zip(snap.nodes.iter()).filter(|(_, s)| s.online).map(|(srv, s)| (srv, s.api_key.clone())).collect();
    for (srv, key) in active_srvs {
        reconcile_folders::reconcile_server_node(srv, &key, &online_ids, &folder_names, store, &mut snap);
    }
    let _ = store.save(&cfg.portal.database_path);
    snap
}
