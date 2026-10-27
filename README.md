<div align="center">
  <img src="assets/icons/icon.svg" alt="THRALL Emblem" width="96" height="96">
  <h1>THRALL</h1>
  <p><strong>Tool for Hierarchical Replication And LDAP Linking</strong></p>
  <p>
    <a href="https://ubermetroid.github.io/thrall/"><img src="https://img.shields.io/badge/docs-GitHub%20Pages-3fb950?style=flat-square" alt="GitHub Pages"></a>
    <a href="https://ubermetroid.github.io/ahamkara/communion.html"><img src="https://img.shields.io/badge/ahamkara-Wish--Dragon%20Communion-a371f7?style=flat-square" alt="Ahamkara Rite of Communion"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-58a6ff?style=flat-square" alt="Apache-2.0 License"></a>
    <img src="https://img.shields.io/badge/dependencies-zero%20(std%20only)-3fb950?style=flat-square" alt="Zero Dependencies">
    <img src="https://img.shields.io/badge/unsafe-forbidden-f85149?style=flat-square" alt="Forbid Unsafe">
  </p>
</div>

---

**THRALL** (`thrall` / `st-mesh-mgr`) is a zero-dependency static Rust control plane and self-service web portal that binds an organization's LDAP directory (RFC 4511) to a distributed multi-server Syncthing fleet.

## Key Capabilities

1. **Option B Per-Top-Level Folder Sync (Zero Dotfiles)**:
   - Never syncs `~/.config` or hidden dotfiles.
   - Provisions each user's top-level folders under `/users/<username>/<Folder>` as individual Syncthing folders (`user-<username>-<folder_lowercase>`), initialized with the 8 standard Linux XDG folders (`Desktop`, `Documents`, `Downloads`, `Music`, `Pictures`, `Public`, `Templates`, `Videos`).
   - Users can create or delete top-level personal folders directly in **Tab 1 (`1. Personal`)**, browse shared team folders in **Tab 2 (`2. Common`)**, and monitor per-Spoke folder acceptance & enrollment in **Tab 3 (`3. Spokes & Setup`)**.
   - Admins can create, delete, and manage per-user LDAP ACLs for top-level organizational shared folders (`/shared/<Folder>`) in **Tab 4 (`4. Access Control`)** and manage regional Mesh Hubs in **Tab 5 (`5. Mesh Hubs`)**.
2. **Resilient Device ID Normalization & Auto-Assigned Labels**:
   - Strips surrounding text, whitespace, hyphens, and invisible Unicode characters when pasting a 56-character Base32 Syncthing Device ID, verifies all 4 Luhn mod 32 check digits, and validates against `/rest/svc/deviceid`.
   - Automatically assigns deterministic device labels (`dev-1-<short_id>`)—no manual label entry required.
3. **Live Anonymous Telemetry Login HUD & Rotating Hive Thrall Wallpapers**:
   - The `/login` screen displays a centered **THRALL — Tool for Hierarchical Replication And LDAP Linking** emblem, a live SVG mesh waveform, an IdleScreen-inspired HUD Box Ticker with anonymous BEP metrics (`NODES`, `CONNS`, `SPEED IN`, `SPEED OUT`, `UPTIME`), and 5 rotating asymmetrical dark Hive Thrall backgrounds (`/bg-1.jpg` .. `/bg-4.jpg`, `/bg.jpg`).
4. **Path-Jailed Web Explorer & Capability Share Links (`/s/<token>`)**:
   - Browse personal and shared top-level folders in the browser with `Share: ON` / `Share: OFF` read-only capability links and strict symlink-escape / dotfile rejection.

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
│   ├── portal.css             # Dark-mode portal & HUD ticker stylesheet
│   ├── thrall-bg.jpg          # Primary Hive Thrall background
│   ├── thrall-bg-1..4.jpg     # 4 asymmetrical off-center Hive Thrall horror backgrounds
│   └── icons/                 # SVG + 16x16..512x512 PNG + multi-res ICO application icons
├── docs/                      # GitHub Pages landing site (https://ubermetroid.github.io/thrall/)
├── guix/                      # GNU Guix reproducible package & container manifest
├── quadlet/                   # Hardened rootless/system Podman systemd Quadlet units
└── thrall-daemon/
    ├── Cargo.toml             # Zero external dependencies (Apache-2.0)
    ├── src/                   # 11 modular pages strictly <= 256 lines each
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
