//! Gate: THRALL ships a control plane, never a peer.
//!
//! A peer is stock Syncthing that the operator runs on every hub. THRALL only
//! ever talks to a peer over that peer's own authenticated REST API, so this
//! repository must not carry a peer image build, a peer unit, or a pipeline
//! that publishes one. Without this gate the peer can quietly creep back in as
//! a second artifact of this project, which is exactly what THRALL must not be.
#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::Path;

/// Artifacts that would turn this repository into a peer distribution.
const PEER_ARTIFACTS: &[&str] = &[
    "Containerfile.syncthing",
    "Containerfile.peer",
    "Containerfile.peer-node",
    "Dockerfile.syncthing",
];

#[test]
fn test_no_peer_artifacts_in_the_repository() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .ok_or("Could not find workspace root")?;

    // No peer image may be built from this repository.
    for rel in PEER_ARTIFACTS {
        assert!(
            !workspace_root.join(rel).exists(),
            "THRALL must not ship a peer image build: {rel}"
        );
    }

    // The quadlet directory carries the control plane and nothing else. A peer
    // unit here would ship the peer as part of this project.
    let quadlet_dir = workspace_root.join("quadlet");
    if quadlet_dir.is_dir() {
        let mut units: Vec<String> = Vec::new();
        for entry in fs::read_dir(&quadlet_dir)?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".container") {
                units.push(name);
            }
        }
        units.sort();
        assert_eq!(
            units,
            vec!["thrall.container".to_string()],
            "the quadlet directory must carry the control-plane unit only: {units:?}"
        );
    }

    // The pipeline must not publish a peer package either.
    let workflow = workspace_root.join(".github/workflows/ci.yml");
    if workflow.exists() {
        let text = fs::read_to_string(&workflow)?;
        for needle in ["IMAGE_PEER", "thrall-syncthing", "Containerfile.syncthing"] {
            assert!(
                !text.contains(needle),
                "the pipeline must not publish a peer image ({needle})"
            );
        }
    }

    Ok(())
}
