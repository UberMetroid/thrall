//! The background reconciliation ticker.
//!
//! The daemon reconciles the fleet on a timer rather than on every request.
//! The interval is administrator-configurable, and maintenance mode pauses the
//! reconciler while still polling the peers so the fleet view stays live.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::device_store::MeshStore;
use crate::fleet_config::FleetConfig;
use crate::reconcile_mesh::{poll_fleet, reconcile_fleet, FleetSnapshot};

/// The daemon state shared between the portal and the ticker.
pub type SharedState = Arc<Mutex<(FleetConfig, MeshStore, FleetSnapshot)>>;

/// What the ticker should do when its timer fires.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Keep waiting; the interval has not elapsed.
    Wait,
    /// Refresh the fleet view without changing anything on a peer.
    Poll,
    /// Reconcile the fleet against the peers.
    Reconcile,
}

/// Decide the ticker's action from the interval, the elapsed count, and the
/// maintenance flag.
pub fn action(interval: u32, elapsed: u32, maintenance: bool) -> Action {
    if elapsed < interval.max(1) {
        return Action::Wait;
    }
    if maintenance {
        Action::Poll
    } else {
        Action::Reconcile
    }
}

/// Start the background reconciler.
pub fn spawn(state: SharedState) {
    std::thread::spawn(move || {
        let mut elapsed: u32 = 0;
        let mut last_interval: u32 = 0;
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let Ok(g) = state.lock() else {
                continue;
            };
            let interval = g.1.settings.interval;
            let maintenance = g.1.settings.maintenance;
            drop(g);
            if interval != last_interval {
                elapsed = 0;
                last_interval = interval;
            }
            elapsed = elapsed.saturating_add(1);
            match action(interval, elapsed, maintenance) {
                Action::Wait => continue,
                Action::Poll => {
                    elapsed = 0;
                    let Some((cfg, mut store)) =
                        state.lock().ok().map(|g| (g.0.clone(), g.1.clone()))
                    else {
                        continue;
                    };
                    let snap = poll_fleet(&cfg, &mut store);
                    if let Ok(mut g) = state.lock() {
                        g.2 = snap;
                    }
                    eprintln!("[thrall] Maintenance mode: polled the fleet, changed nothing");
                }
                Action::Reconcile => {
                    elapsed = 0;
                    let Some((cfg, mut store)) =
                        state.lock().ok().map(|g| (g.0.clone(), g.1.clone()))
                    else {
                        continue;
                    };
                    let snap = reconcile_fleet(&cfg, &mut store);
                    if let Ok(mut g) = state.lock() {
                        g.2 = snap;
                    }
                    eprintln!("[thrall] Reconciled the fleet");
                }
            }
        }
    });
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_the_ticker_waits_until_the_interval_elapses() {
        assert_eq!(action(45, 44, false), Action::Wait);
        assert_eq!(action(45, 45, false), Action::Reconcile);
    }

    #[test]
    fn test_maintenance_mode_polls_instead_of_reconciling() {
        assert_eq!(action(45, 45, true), Action::Poll);
        assert_eq!(action(45, 10, true), Action::Wait);
    }

    #[test]
    fn test_a_zero_interval_still_waits_one_tick() {
        assert_eq!(action(0, 0, false), Action::Wait);
        assert_eq!(action(0, 1, false), Action::Reconcile);
    }
}
