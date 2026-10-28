//! mesh_stage.rs — Hub selection and per-hub telemetry series for the mesh stage.
//!
//! Vocabulary: a **hub** is a configured fleet server, a **device** is a paired
//! user workstation. The mesh stage draws hubs only; a hub that is offline is
//! absent from the stage, never rendered as a dead node.
//!
//! Roster order is preserved everywhere, so a hub keeps its slot identity for as
//! long as it stays online, and the parallel telemetry series stay index-aligned.

use crate::reconcile_mesh::{FleetSnapshot, NodeStatus};

/// Comma-joined per-hub telemetry series.
pub struct HubSeries {
    pub rtts: String,
    pub bps: String,
    pub masses: String,
    pub bearings: String,
}

/// Hubs currently online, in roster order — exactly what the mesh stage draws.
pub fn online_hubs(snap: &FleetSnapshot) -> Vec<&NodeStatus> {
    snap.nodes.iter().filter(|n| n.online).collect()
}

/// Roster-ordered online flags, one per configured hub.
pub fn online_flags(snap: &FleetSnapshot) -> Vec<bool> {
    snap.nodes.iter().map(|n| n.online).collect()
}

/// Roster-ordered online flags as a CSV literal, e.g. `true,false,true`.
pub fn online_flags_csv(snap: &FleetSnapshot) -> String {
    online_flags(snap)
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// Per-hub series for the given hubs, with synthetic fallbacks for missing telemetry.
pub fn hub_series(hubs: &[&NodeStatus]) -> HubSeries {
    let (mut rtts, mut bps, mut masses, mut bearings) =
        (String::new(), String::new(), String::new(), String::new());
    for (idx, n) in hubs.iter().enumerate() {
        let i = idx as u64;
        let rtt = if n.rtt_ms > 0 { n.rtt_ms } else { (i + 1) * 14 };
        let mass = if n.total_bytes > 0 {
            n.total_bytes
        } else {
            (i + 1) * 1024
        };
        if !rtts.is_empty() {
            rtts.push(',');
            bps.push(',');
            masses.push(',');
            bearings.push(',');
        }
        rtts.push_str(&rtt.to_string());
        bps.push_str(&(n.in_bps + n.out_bps).to_string());
        masses.push_str(&mass.to_string());
        bearings.push_str(&n.bearing_deg.to_string());
    }
    HubSeries {
        rtts,
        bps,
        masses,
        bearings,
    }
}

/// Parent hub index per device, in roster indexing.
///
/// A device whose hub has gone offline keeps pointing at its roster slot; the
/// stage re-homes it onto a live hub when it resolves the roster into slots.
pub fn device_parents(snap: &FleetSnapshot, device_ids: &[String]) -> String {
    let roster_total = snap.nodes.len().max(1);
    device_ids
        .iter()
        .enumerate()
        .map(|(di, did)| {
            snap.nodes
                .iter()
                .position(|n| n.connected_ids.contains(did))
                .map(|p| p.to_string())
                .unwrap_or_else(|| (di % roster_total).to_string())
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{device_parents, hub_series, online_flags, online_hubs};
    use crate::reconcile_mesh::{FleetSnapshot, NodeStatus};

    fn hub(name: &str, online: bool, rtt: u64, mass: u64, devices: &[&str]) -> NodeStatus {
        NodeStatus {
            name: name.into(),
            api_url: String::new(),
            wan_addr: String::new(),
            online,
            disabled: !online,
            device_id: String::new(),
            api_key: String::new(),
            uptime_sec: 0,
            active_conns: 0,
            in_bps: 0,
            out_bps: 0,
            rx_bytes: 0,
            tx_bytes: 0,
            total_bytes: mass,
            rtt_ms: rtt,
            bearing_deg: 0,
            connected_ids: devices.iter().map(|d| (*d).to_string()).collect(),
            last_error: String::new(),
        }
    }

    #[test]
    fn offline_hubs_are_excluded_from_the_stage() {
        let snap = FleetSnapshot {
            nodes: vec![
                hub("hub-a", true, 1, 4096, &[]),
                hub("hub-c", false, 0, 0, &[]),
            ],
            ..FleetSnapshot::default()
        };
        let hubs = online_hubs(&snap);
        assert_eq!(hubs.len(), 1);
        assert_eq!(hubs[0].name, "hub-a");
        assert_eq!(online_flags(&snap), vec![true, false]);
    }

    #[test]
    fn series_are_index_aligned_and_skip_offline_hubs() {
        let snap = FleetSnapshot {
            nodes: vec![
                hub("hub-a", true, 1, 4096, &[]),
                hub("hub-c", false, 0, 0, &[]),
            ],
            ..FleetSnapshot::default()
        };
        let series = hub_series(&online_hubs(&snap));
        assert_eq!(series.rtts, "1");
        assert_eq!(series.masses, "4096");
        assert_eq!(series.bps, "0");
    }

    #[test]
    fn device_parents_point_at_their_roster_hub() {
        let snap = FleetSnapshot {
            nodes: vec![
                hub("hub-a", true, 1, 4096, &["DEV-A"]),
                hub("hub-c", false, 0, 0, &["DEV-B"]),
            ],
            ..FleetSnapshot::default()
        };
        let ids = vec!["DEV-A".to_string(), "DEV-B".to_string()];
        assert_eq!(device_parents(&snap, &ids), "0,1");
    }

    #[test]
    fn login_stage_advertises_only_the_online_hubs() {
        let snap = FleetSnapshot {
            nodes: vec![
                hub("hub-a", true, 1, 4096, &[]),
                hub("hub-b", true, 1, 2048, &[]),
                hub("hub-c", false, 0, 0, &[]),
            ],
            ..FleetSnapshot::default()
        };
        let store = crate::device_store::MeshStore::default();
        let html = crate::render_portal::render_login_page(None, &snap, &store);
        assert!(html.contains("data-hubs-online=\"2\""));
        assert!(html.contains("data-hubs-total=\"3\""));
        assert!(html.contains("data-hubs-flags=\"true,true,false\""));
        assert!(!html.contains("data-mesh-nodes="));
    }
}
