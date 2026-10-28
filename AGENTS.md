# thrall (`st-mesh-mgr`) — House Laws & Agent Engineering Standards

This document is the **single canonical source of truth** for all code, architecture, and system integration standards for `thrall` (**T**ool for **H**ierarchical **R**eplication **A**nd **I**dentity **L**inking — Multi-Server Syncthing LDAP Control Plane & Self-Service Portal). Every human contributor and AI agent must strictly follow these rules without exception.

**Precedence:** Where this file and CI disagree, this file wins.

**Rule Markers:**
- `[GATE:<tool>]` — Machine-enforced by automated compiler flags, clippy, or `cargo test`.
- `[POLICY]` — Enforced during code review. Dictates architecture, naming taste, restraint, and ergonomics.

---

## 1. The Page Rule (Code Layout & Sizing)

A **page** is one committed Rust file. Every page holds one idea, fits in one head, and carries its own weight.

- `[GATE:test]` **16–256 Lines**: Every committed `.rs` file must be between **16 and 256 lines** (`content.lines().count()`, blank lines and comments count). Enforced by `tests/page_rules.rs`.
- `[GATE:test]` **Shim Exemption (Floor Only)**: A file is a shim if every non-empty, non-comment, non-attribute line starts with `mod `, `use `, `pub mod `, or `pub use `. Shims skip the 16-line floor; the 256-line ceiling still strictly applies.
- `[GATE:test]` **Directory Density (≤ 8 files)**: At most 8 `.rs` files per directory.
- `[GATE:test]` **Banned Drawer Filenames**: `util.rs`, `utils.rs`, `helper.rs`, `helpers.rs`, `common.rs`, `misc.rs`, `shared.rs`, `base.rs`, `core.rs`.
- `[POLICY]` **Domain Naming Recipe**:
  - Action pages: `ldap_bind.rs`, `peer_client.rs`, `reconcile_mesh.rs`, `render_portal.rs`.
  - State pages: `fleet_config.rs`, `device_store.rs`, `portal_server.rs`.
- `[POLICY]` **Blobs Out**: Move static configurations and stylesheets into `assets/` and load with `include_str!`.

---

## 2. Architecture & Zero-Dependency Discipline

- `[GATE:test]` **Zero External Dependencies**: `thrall-daemon` compiles with zero external crates in `[dependencies]` (`std` only) for minimal memory footprint, zero supply-chain bloat, and instant reproducible builds.
- `[POLICY]` **Deep Modules over Speculative Traits (Rule of Two)**: Prefer concrete structs with cohesive methods over speculative traits.

---

## 3. Pure Rust & Crash-Resilience Standards

- `[GATE:clippy]` **Zero-Panic Invariant**: No bare `.unwrap()`, `.expect()`, or `panic!()` in production code (`#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`). Permitted **only** inside `#[cfg(test)]` modules.
- `[GATE:rustc]` **`#![forbid(unsafe_code)]`**: Unsafe code is strictly forbidden across all crate roots and test modules.
- `[GATE:test]` **No Shell Invocations**: Subprocesses must never invoke `sh -c` or `bash -c`. Use direct `std::process::Command::new(argv0).args(...)`. Enforced by `tests/lint.rs`.

---

## 4. Verification Ladder

Order matters. Each step must pass completely before running the next:

1. `cargo test --workspace --all-targets` — All unit, integration, and Page Rule lint tests pass.
2. `cargo clippy --workspace --all-targets -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic` — Zero warnings, zero panics.
3. `cargo fmt --all -- --check` — Standard formatting verified.
4. `cargo build --release --workspace` — Stripped release binaries built cleanly.
