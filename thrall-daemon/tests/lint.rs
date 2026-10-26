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
    assert!(containerfile.contains("syncthing"));
    assert!(containerfile.contains("VOLUME [\"/config\", \"/users\", \"/shared\"]"));

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
    let banned_hashes: [u64; 18] = [
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
