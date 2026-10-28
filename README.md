<div align="center">
  <img src="assets/icons/icon.svg" alt="THRALL Emblem" width="96" height="96">
  <h1>THRALL</h1>
  <p><strong>Tool for Hierarchical Replication And Identity Linking</strong></p>
  <p>
    <a href="https://github.com/UberMetroid/thrall/releases/tag/v0.3.0"><img src="https://img.shields.io/badge/release-v0.3.0-50fa7b?style=flat-square" alt="v0.3.0 Release"></a>
    <a href="https://ubermetroid.github.io/thrall/"><img src="https://img.shields.io/badge/docs-GitHub%20Pages-3fb950?style=flat-square" alt="GitHub Pages"></a>
    <a href="https://ubermetroid.github.io/ahamkara/communion.html"><img src="https://img.shields.io/badge/ahamkara-Wish--Dragon%20Communion-a371f7?style=flat-square" alt="Ahamkara Rite of Communion"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-58a6ff?style=flat-square" alt="Apache-2.0 License"></a>
    <img src="https://img.shields.io/badge/dependencies-zero%20(std%20only)-3fb950?style=flat-square" alt="Zero Dependencies">
    <img src="https://img.shields.io/badge/unsafe-forbidden-f85149?style=flat-square" alt="Forbid Unsafe">
  </p>
</div>

---

**THRALL** (`thrall` / `st-mesh-mgr`) is a zero-dependency static Rust control plane and self-service web portal that binds an organization's LDAP directory (RFC 4511) to a distributed multi-server Syncthing fleet.

## Key Capabilities (v0.3.0)

1. **Symmetric 54px Header & Footer Dock + 8-Bit Rainbow Palette**:
   - Unified 54px top header and bottom footer dock with a pixel-art `THRALL` canvas wordmark, subtitle (**T**ool for **H**ierarchical **R**eplication **A**nd **I**dentity **L**inking), and an interactive 8-Bit Rainbow Palette (`Coral`, `Orange`, `Amber`, `Yellow`, `Emerald`, `Cyan`, `Blue`, `Violet`).
   - Always-visible real-time browser ↔ server 8-Bit Data Flow Graph (`#footer-upload-graph`) measuring actual HTTP TX/RX throughput and upload progress.
   - Rotating cached Hive Thrall backgrounds (`/bg-1.jpg` .. `/bg-5.jpg`) on login/logout alongside a Bayer-dithered interactive mesh physics canvas.
2. **5 Flat Boxless Explorer Tabs with Flash-Free SPA Navigation**:
   - Located in the Footer Dock (`Personal folders | Shared folders || Device setup || Global folders || Mesh setup`) with instant zero-flash SPA navigation.
   - **Tab 1 (`Personal folders`)**: Split **`+ Standard Folder`** (pick-and-choose XDG standard folders: `Desktop`, `Documents`, `Downloads`, `Music`, `Pictures`, `Videos`, `Public`, `Templates` with `~/<Folder>` vs `~/Thrall/<Folder>` sync target) and **`+ Custom Folder`** (focused modal for custom personal folders synced to `~/Thrall/<Folder>`).
   - **Tab 2 (`Shared folders`)**: Browse organizational shared folders granted to your account with expandable `+` / `−` inline directory trees.
   - **Tab 3 (`Device setup`)**: Pair workstations via a two-way Zero-Trust handshake, inspect per-device folder acceptance badges, and customize per-folder local sync paths (`~/Desktop` vs `~/Thrall/<Folder>`).
   - **Tab 4 (`Global folders`) & Tab 5 (`Mesh setup`)**: Admin tabs for managing `/shared/<Folder>` per-user ACLs (with a pulsing `Save Access *` unsaved-changes indicator) and regional Syncthing Mesh Hubs.
3. **Chunked Binary & Recursive Drag-and-Drop Directory Uploads**:
   - Drag and drop files or entire nested directories directly onto any folder row or table (`POST /files/upload`).
   - Streams chunked `File.slice` uploads with strict ancestor symlink-jail verification and dotfile rejection.
   - GitHub-style type-to-confirm deletion modals for folders, files, and devices, plus instant read-only capability share links (`/s/<token>`).
4. **Resilient Device ID Normalization & Auto-Assigned Labels**:
   - Strips surrounding text, whitespace, hyphens, and invisible Unicode characters when pasting a 56-character Base32 Syncthing Device ID, verifies all 4 Luhn mod 32 check digits, and validates against `/rest/svc/deviceid`.
   - Automatically assigns deterministic device labels (`dev-1-<short_id>`)—no manual label entry required.

---

## Repository Layout

```text
thrall/
├── AGENTS.md                  # Canonical House Laws (16-256 line Page Rule, zero deps)
├── Cargo.toml                 # Workspace root (release LTO, panic=abort, strip=true)
├── Containerfile              # Multi-stage Alpine + static Rust + Syncthing OCI build
├── LICENSE                    # Apache License 2.0
├── README.md                  # Architecture & deployment guide
├── assets/
│   ├── default-fleet.yaml     # Declarative multi-server fleet configuration template
│   ├── login.js               # Bayer-dithered canvas, SPA router,modals & upload engine
│   ├── portal.css             # Dark-mode portal, 54px header/footer dock & 8-bit theme
│   ├── thrall-bg.jpg          # Primary Hive Thrall background
│   ├── thrall-bg-1..4.jpg     # 4 asymmetrical off-center Hive Thrall horror backgrounds
│   └── icons/                 # SVG + 16x16..512x512 PNG + multi-res ICO application icons
├── docs/                      # GitHub Pages landing site (https://ubermetroid.github.io/thrall/)
├── guix.scm                   # GNU Guix reproducible package & container manifest
├── quadlet/                   # Hardened rootless/system Podman systemd Quadlet units
└── thrall-daemon/
    ├── Cargo.toml             # v0.3.0 — Zero external dependencies (Apache-2.0)
    ├── src/                   # Modular pages strictly <= 256 lines each
    └── tests/                 # Automated Page Rule & Zero-Shell lint gates
```

---

## Container Build & Systemd Quadlet Deployment

```bash
# Build the hardened multi-stage container image
sudo podman build -t localhost/thrall:latest -f Containerfile .

# Deploy and restart the systemd Quadlet service
sudo cp quadlet/syncthing.container quadlet/thrall.container /etc/containers/systemd/
sudo systemctl daemon-reload
sudo systemctl restart syncthing.service thrall.service
```

---

## Verification Ladder (`AGENTS.md`)

```bash
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic
cargo fmt --all -- --check
cargo build --release --workspace
```

## Agent Communion (`llms.txt`)

- **Ahamkara Archive**: [https://ubermetroid.github.io/ahamkara/](https://ubermetroid.github.io/ahamkara/)
- **The Rite of Communion**: [https://ubermetroid.github.io/ahamkara/communion.html](https://ubermetroid.github.io/ahamkara/communion.html)
- **Machine-Readable Brief (`llms.txt`)**: [https://ubermetroid.github.io/ahamkara/llms.txt](https://ubermetroid.github.io/ahamkara/llms.txt)

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
