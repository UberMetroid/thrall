//! Workspace-wide lint tests ensuring zero shell invocations ("sh -c", "bash -c", "zsh -c")
//! and validating Containerfile and Quadlet consistency.
#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name != "target" && name != ".git" && name != "prebuilt" {
                collect_rs_files(&path, files);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[test]
fn test_no_shell_invocations_in_workspace() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .ok_or("Could not find workspace root")?;

    let mut rs_files = Vec::new();
    collect_rs_files(workspace_root, &mut rs_files);
    assert!(!rs_files.is_empty(), "No .rs files found to lint");

    for file in rs_files {
        if file.ends_with("tests/lint.rs") {
            continue;
        }
        let content = fs::read_to_string(&file)?;

        let forbidden_patterns = [
            ("Command::new(", "\"sh\")"),
            ("Command::new(", "\"/bin/sh\")"),
            ("Command::new(", "\"bash\")"),
            ("Command::new(", "\"/bin/bash\")"),
            ("Command::new(", "\"zsh\")"),
            ("Command::new(", "\"/bin/zsh\")"),
        ];
        for (prefix, suffix) in forbidden_patterns {
            let pat = format!("{prefix}{suffix}");
            assert!(
                !content.contains(&pat),
                "Forbidden shell invocation {pat} detected in {}",
                file.display()
            );
        }

        let has_dash_c = content.contains("\"-c\"");
        let has_shell_bin = content.contains("\"sh\"")
            || content.contains("\"bash\"")
            || content.contains("\"zsh\"");
        assert!(
            !(has_dash_c && has_shell_bin),
            "Forbidden shell -c invocation detected in {}",
            file.display()
        );
    }
    Ok(())
}

#[test]
fn test_containerfile_and_quadlet_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .ok_or("Could not find workspace root")?;

    let containerfile = fs::read_to_string(workspace_root.join("Containerfile"))?;
    assert!(containerfile.contains("/bin/thrall"));
    assert!(!containerfile.contains("ln -s"));
    assert!(containerfile.contains("VOLUME [\"/config\", \"/users\", \"/shared\"]"));

    // The control-plane image must never bundle a Syncthing peer. Peers live in
    // their own container (the thrall-syncthing image) and the daemon only ever
    // talks to them over their REST API.
    assert!(
        !containerfile
            .lines()
            .filter(|l| l.trim_start().to_uppercase().starts_with("FROM "))
            .any(|l| l.contains("syncthing")),
        "the thrall control-plane image must not have a syncthing build stage"
    );
    assert!(
        !containerfile.lines().any(|l| {
            let t = l.trim_start().to_lowercase();
            t.contains("install") && t.contains("syncthing")
        }),
        "the thrall control-plane image must not install a syncthing package"
    );

    // The control-plane daemon must never spawn or reference a local Syncthing
    // binary either: no env-var opt-in, no binary path, no spawn helper.
    //
    // The same rule covers SSH. THRALL reaches every peer over that peer's own
    // authenticated REST API, so there is nothing left for an SSH client to do.
    // A fleet-wide SSH key can reach every hub at once; a per-peer API key is
    // scoped to one peer's REST surface. The image must not even carry one.
    let mut daemon_files = Vec::new();
    collect_rs_files(workspace_root, &mut daemon_files);
    for file in &daemon_files {
        if file.ends_with("tests/lint.rs") {
            continue;
        }
        let content = fs::read_to_string(file)?;
        for needle in [
            "THRALL_LOCAL_SYNCTHING",
            "/bin/syncthing",
            "/usr/bin/syncthing",
            "maybe_spawn_local_syncthing",
            "Command::new(\"ssh\")",
            "openssh",
            "/usr/bin/ssh",
            "/bin/ssh",
            "ssh_host",
            "ssh_user",
            "ssh_key_path",
            "mascot.jpg",
        ] {
            assert!(
                !content.contains(needle),
                "the thrall daemon must not reference a local Syncthing binary or an SSH \
                 client ({needle}) in {}",
                file.display()
            );
        }
    }

    // The image must not install an SSH client either.
    assert!(
        !containerfile.lines().any(|l| {
            let t = l.trim().to_lowercase();
            t.contains("microdnf install") && t.contains("openssh")
        }),
        "the thrall control-plane image must not install an SSH client"
    );

    // The shipped fleet template must not carry SSH settings.
    let fleet = fs::read_to_string(workspace_root.join("assets/default-fleet.yaml"))?;
    assert!(
        !fleet.contains("ssh_"),
        "the default fleet configuration must not carry SSH settings"
    );

    // The archived mascot image belongs to a project THRALL does not ship. It
    // must not be restored to the image or to the published page.
    for rel in ["assets/thrall-mascot.jpg", "docs/mascot.jpg"] {
        assert!(
            !workspace_root.join(rel).exists(),
            "the archived mascot image must not be restored: {rel}"
        );
    }

    let quadlet = fs::read_to_string(workspace_root.join("quadlet/thrall.container"))?;
    assert!(quadlet.contains("NoNewPrivileges=true"));
    assert!(quadlet.contains("DropCapability=ALL"));

    let mut all_files = Vec::new();
    collect_rs_files(workspace_root, &mut all_files);
    for rel in [
        "assets/login.js",
        "assets/portal.css",
        "assets/default-fleet.yaml",
        "README.md",
        "AGENTS.md",
        "docs/index.html",
        "quadlet/thrall.container",
        "quadlet/syncthing.container",
    ] {
        all_files.push(workspace_root.join(rel));
    }
    let banned_hashes: [u64; 24] = [
        0xccce_1e1b_c85b_3879,
        0x0e6b_f4fe_abe1_d637,
        0xf6e0_5d31_6661_9706,
        0x6843_004d_5982_e419,
        0xc1d8_8c8b_ccd4_39c2,
        0xb407_f1b4_faea_9663,
        0x2a3a_dce6_0eea_798a,
        0xc31d_b242_d96e_6df2,
        0xcd07_a5ea_5c42_3991,
        0xe3b0_3bf2_d76f_366c,
        0x787a_39d7_6e6f_53ae,
        0x03ab_7db6_e176_e57a,
        0x4d0e_6e1f_bfff_55fa,
        0x3c7e_df27_e8e6_9c51,
        0xa0e7_6a0b_66bb_e51c,
        0x3622_2b99_6a0c_e604,
        0x5f5d_1c20_885b_c3bd,
        0x8ca1_e9d0_63ee_114f,
        // The external archive the project used to link to. It is not part of
        // THRALL, so none of its names may appear in a published surface.
        0x8baa_8108_53ce_3d7b,
        0x817f_b5b0_59a3_d5d0,
        0x7104_93a2_1d69_315e,
        0x037e_2b0d_eb9b_006d,
        0xbe36_4638_08f3_e590,
        0xea9e_60ad_81da_16f1,
    ];
    for f in all_files {
        let Ok(txt) = fs::read_to_string(&f) else {
            continue;
        };
        for word in txt
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|w| w.len() >= 4)
        {
            let mut h = 0xcbf2_9ce4_8422_2325u64;
            for b in word.bytes().map(|b| b.to_ascii_lowercase()) {
                h = (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
            }
            assert!(
                !banned_hashes.contains(&h),
                "Banned PII token '{}' detected in {}",
                word,
                f.display()
            );
        }
    }

    Ok(())
}
