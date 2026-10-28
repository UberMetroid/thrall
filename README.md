# THRALL

**Tool for Hierarchical Replication And Identity Linking**

**THRALL turns your LDAP directory into a file-sync fleet.** Staff sign in with the work account they already have, and their folders are already there — on every machine they own. Team folders are granted and revoked from one screen.

[![CI](https://github.com/UberMetroid/thrall/actions/workflows/ci.yml/badge.svg)](https://github.com/UberMetroid/thrall/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/release-v0.5.3-50fa7b?style=flat-square)](https://github.com/UberMetroid/thrall/releases/tag/v0.5.3)
[![License](https://img.shields.io/badge/license-Apache--2.0-58a6ff?style=flat-square)](LICENSE)
[![Zero dependencies](https://img.shields.io/badge/dependencies-zero%20(std%20only)-3fb950?style=flat-square)](https://github.com/UberMetroid/thrall/blob/main/AGENTS.md)
[![No unsafe](https://img.shields.io/badge/unsafe-forbidden-f85149?style=flat-square)](https://github.com/UberMetroid/thrall/blob/main/AGENTS.md)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-3fb950?style=flat-square)](https://ubermetroid.github.io/thrall/)

**[Landing page](https://ubermetroid.github.io/thrall/)** · **[Releases](https://github.com/UberMetroid/thrall/releases)** · **[House Laws](https://github.com/UberMetroid/thrall/blob/main/AGENTS.md)**

---

## The problem it removes

Handing a team a synced share usually means:

- installing a sync client on every laptop,
- sharing each folder by hand, device by device,
- and un-sharing everything by hand when someone leaves.

Your directory already knows who works here and who no longer does. THRALL reads that and does the work for you.

## What the person using it actually does

**1. Signs in with their work account.** The portal binds directly to your LDAP or Active Directory server over RFC 4511. Authentication is the directory's job — THRALL never stores a user password, and there is no account to create.

**2. Finds their folders already there.** Standard folders (`Desktop`, `Documents`, `Music`, `Pictures`, `Videos`) and any custom folder are provisioned on the hub and bound to their enrolled devices. Each folder can have its own local path on each workstation.

**3. Gets access from one screen.** An admin ticks a person against a team folder. It reaches every device that person owns within seconds. Untick it and access is revoked everywhere, including the devices you cannot reach.

Workstations never expose their own REST API and never connect to each other — only to the hub.

## How a fleet is laid out

THRALL is a **control plane**, never a peer. It never bundles, installs, or spawns Syncthing, and this project publishes no peer image. The peer — the thing that actually stores and replicates bytes — is stock Syncthing that you run on every hub that holds data.

```text
    +----------------------------------------------+
    |         Control-plane host  (ONE per fleet)  |
    |                                              |
    |         [ thrall container ]                 |
    |           - RFC 4511 LDAP bind               |
    |           - self-service web portal          |
    |           - roster / folder topology / ACLs  |
    |           - reconciler: calls each peer's    |
    |             authenticated REST API           |
    |           - no SSH client, no SSH access     |
    +--------+---------------+---------------+-----+
             |               |               |
         REST API        REST API        REST API
        (loopback)       (tailnet)       (tailnet)
             |               |               |
             v               v               v
      +------------+  +------------+  +------------+
      |   hub-a    |  |   hub-b    |  |   hub-c    |
      | [ syncthing|  | [ syncthing|  | [ syncthing|
      |  container]|  |  container]|  |  container]|
      +------------+  +------------+  +------------+
```

### One control plane per fleet, not one per hub

The roster (users, folder topology, access rules, device enrollments) is a single-writer store: it is saved as a whole file with no lock, no version counter, and no merge. Every control plane reconciles *every* hub listed in `fleet.yaml`.

Running one control plane per hub would therefore mean `N × N` reconciliation writes racing into the same peers, plus `N` portals and `N` credential sets to defend. That is a duplicated brain, not redundancy.

For failover, run a **warm standby**: a second control plane holding a standby copy of the roster, promoted deliberately, never reconciling concurrently with the primary.

### How a peer is reached

Every peer is reached over **its own REST API**, authenticated with **that peer's API key**.

| Peer | `local` | Address | Reached over |
|---|---|---|---|
| Shares the host with the daemon | `true` | `http://127.0.0.1:8384` | loopback |
| On another host | `false` | e.g. `http://hub-b:8384` | a reachable address, such as a private overlay network |

The peer creates its own folder paths when the folder configuration is applied, so the daemon never touches a peer's filesystem.

The peer is software you run yourself: THRALL publishes no peer image and ships no peer unit. See [Running a peer](#running-a-peer).

There is **no SSH anywhere in the path** — not in the control-plane image, not in the fleet configuration, and no fleet-wide credential that can reach every hub at once. A per-peer API key is scoped to one peer's REST surface.

## Why you can trust it with a directory server

THRALL holds an LDAP bind password and the access rules for every user's files. These are the claims this project makes, and the test that fails the build if a claim stops being true.

| Claim | How it is enforced |
|---|---|
| Zero external crates — the standard library only | `tests/page_rules.rs` fails the build if `[dependencies]` is ever non-empty |
| No unsafe code | `#![forbid(unsafe_code)]` at the crate root; the compiler rejects it |
| No shell, ever | `tests/lint.rs` fails if any code path spawns `sh`, `bash`, or `zsh -c` |
| No SSH client in the image | `tests/lint.rs` fails if the `Containerfile` installs one, or the daemon references `ssh` |
| No Syncthing in the control plane | `tests/lint.rs` fails if the image gains a Syncthing stage, package, or binary |
| No peer image is published | `tests/no_peer_artifacts.rs` fails if a peer image, a peer unit, or a peer publish step reappears |
| The session secret fails closed | the daemon refuses the shipped placeholder and any secret under 16 characters, and generates one from `/dev/urandom` if empty |
| Share links cannot be forged | a keyed 128-bit MAC over the server secret, not a hash of the path |
| Peer credentials are scoped per peer | each peer has its own API key; a remote peer pointed at loopback is refused at discovery |
| Credential files are owner-only | every write creates the file `0600` and renames it into place |
| The daemon never panics on bad input | clippy denies `unwrap`, `expect`, and `panic` outside test code |

## What it costs to run

- **One static binary.** No runtime services, no database server, no message queue, no cache.
- **One container image** — the control plane, with no package manager in the runtime layer.
- **No peer to maintain.** The peer is stock Syncthing; this project publishes no peer image and no peer unit.
- **256 MB** memory ceiling on the control plane, single process.

---

## Getting started

### Prerequisites

- A directory server reachable over LDAP (RFC 4511) — OpenLDAP, Active Directory, or compatible.
- One host that will run the **control plane**, plus one host per hub that will store files.
- Podman with systemd Quadlet support, on every host above.
- A private network the peers can reach each other on (the peer-to-peer protocol uses TLS 1.3).
- **A Syncthing peer on every hub that stores files.** THRALL ships no peer: any stock Syncthing works, and this project does not build or publish one. See [Running a peer](#running-a-peer).

### 1. Get the image

```bash
sudo podman pull ghcr.io/ubermetroid/thrall:0.5.3
```

Or build it from this repository:

```bash
sudo podman build -t localhost/thrall:0.5.3 -f Containerfile .
```

If you build locally, point the unit's `Image=` line at your local tag. This is the only image this project publishes.

### 2. Give the container the directory bind password

The bind password is a container secret, never a file on disk and never a line in `fleet.yaml`:

```bash
printf '%s' '<bind password>' | sudo podman secret create ldap_bind_password -
```

### 3. Deploy the unit

```bash
sudo cp quadlet/thrall.container /etc/containers/systemd/
sudo systemctl daemon-reload
sudo systemctl enable --now thrall.service
```

The unit runs as an unprivileged user, drops all capabilities, mounts the root filesystem read-only, and exposes only the directories it needs. It sets `PORTAL_LISTEN_ADDR=0.0.0.0:8088`; the configuration file's own default is `0.0.0.0:8080`.

Run a peer on every hub that stores files — see [Running a peer](#running-a-peer).

### 4. Configure the fleet

The control plane reads `/config/fleet.yaml`. Start from `assets/default-fleet.yaml`:

```yaml
portal:
  listen_addr: "0.0.0.0:8080"
  database_path: "/config/roster.db"
  # Leave empty: the daemon generates a random secret and persists it here on first
  # boot. The daemon refuses to start while this still holds the shipped placeholder.
  session_secret: ""

ldap:
  url: "ldap://ldap.example.com:389"
  base_dn: "dc=example,dc=com"
  bind_dn: "cn=admin,dc=example,dc=com"
  bind_password: ""
  user_filter: "(&(objectClass=person)(uid=%s))"
  admin_users: ["admin"]

servers:
  - name: "hub-a"
    local: true
    syncthing_api_url: "http://127.0.0.1:8384"
    syncthing_api_key: "<the peer's API key>"
    syncthing_wan_addr: "tcp://hub-a.example.com:22000"
```

### 5. Give each peer its API key

The API key authenticates THRALL against that peer. Read it once from the peer's own configuration, on the peer host:

```bash
sudo podman exec syncthing sh -c \
  'grep -oE "<apikey>[^<]+" /var/syncthing/config/config.xml | head -1 | cut -d">" -f2'
```

### 6. Publish a remote peer's API

A peer binds its API to loopback. To reach it from the control plane, publish it on your private network — never on the public internet:

```bash
# On the peer host
sudo tailscale serve --bg --http=8384 http://127.0.0.1:8384
```

Tailscale Serve routes by hostname, so the fleet entry must use the peer's name rather than its raw address:

```yaml
servers:
  - name: "hub-b"
    local: false
    syncthing_api_url: "http://hub-b:8384"
    syncthing_api_key: "<the peer's API key>"
    syncthing_wan_addr: "tcp://100.64.0.20:22000"
```

The peer-to-peer address keeps using the raw address: port 22000 is published by the peer itself and does not pass through Serve.

### 7. Enroll a device

From the portal's **Devices** tab, an admin adds the workstation:

1. **On the workstation**, add the hub's device ID to the local Syncthing as an introducer with auto-accept enabled.
2. **In the portal**, paste the workstation's 56-character device ID. THRALL verifies all four Base32 Luhn check digits, names the device automatically, and tracks which folders each device has accepted.

### Running a peer

A peer is stock Syncthing. THRALL does not build, publish, or configure one for you — it only ever talks to a peer's REST API. The upstream image is a drop-in:

```bash
sudo podman pull syncthing/syncthing:2.1.7
```

Two things matter, and both are about keeping the peer's identity stable:

1. **Persist the peer's config directory.** `STHOMEDIR` must point at the bind-mounted directory that holds `config.xml`. That file carries the peer's device ID, its API key, and its TLS certificate. Without it the peer falls back to an anonymous volume and comes back as a brand-new device that no other hub trusts.
2. **Bind the REST API to loopback.** Publish it on your private network if the control plane is on another host — never on the public internet.

A peer that satisfies both, in Quadlet form:

```ini
[Container]
Image=syncthing/syncthing:2.1.7
User=1000:1000
Environment=STHOMEDIR=/var/syncthing/config
Environment=STGUIADDRESS=0.0.0.0:8384
Environment=STNOUPGRADE=1
PublishPort=127.0.0.1:8384:8384/tcp
PublishPort=22000:22000/tcp
PublishPort=22000:22000/udp
Volume=/srv/syncthing/config:/var/syncthing/config:rw,Z
ReadOnly=true
NoNewPrivileges=true
DropCapability=ALL
```

Anything that satisfies the two rules above will do — a systemd unit, a compose file, a bare `podman run`. THRALL needs only the peer's REST API and its API key.

---

## Configuration reference

### `fleet.yaml`

| Key | Meaning |
|---|---|
| `portal.listen_addr` | Address the portal listens on. Default `0.0.0.0:8080`. |
| `portal.database_path` | Where the roster is stored. Default `/config/roster.db`. |
| `portal.session_secret` | Signs session cookies. Leave empty and the daemon generates one from `/dev/urandom` and persists it. The shipped placeholder and any secret under 16 characters are refused at startup. |
| `ldap.url` | LDAP server URI, for example `ldap://ldap.example.com:389`. |
| `ldap.base_dn` | Search base, for example `dc=example,dc=com`. |
| `ldap.bind_dn` | Account used to search the directory. |
| `ldap.bind_password` | Its password. Prefer the `LDAP_BIND_PASSWORD` secret over this field. |
| `ldap.user_filter` | Filter that resolves a login name to a directory entry. |
| `ldap.admin_users` | Login names that get the Admin tab. |
| `servers[].name` | Hub name. Lowercase, unique. |
| `servers[].local` | `true` when the peer shares the host with the daemon. THRALL then calls it over loopback and prepares the folder directories itself. |
| `servers[].syncthing_api_url` | Base URL of the peer's REST API. A remote peer must not point at loopback — that is refused at discovery. |
| `servers[].syncthing_api_key` | The peer's API key. Required for every peer. |
| `servers[].syncthing_wan_addr` | The peer's peer-to-peer address, advertised to other peers. |

### Environment overrides

Any of these takes precedence over the file:

`PORTAL_LISTEN_ADDR` · `PORTAL_DATABASE_PATH` · `PORTAL_SESSION_SECRET` · `LDAP_URL` · `LDAP_BASE_DN` · `LDAP_BIND_DN` · `LDAP_BIND_PASSWORD` · `LDAP_USER_FILTER` · `ADMIN_USERS`

### Command line

```
thrall [--config <path>]   # run the daemon (default: /config/fleet.yaml)
thrall --check             # health probe; exits non-zero if the portal is not serving
thrall --reconcile-once    # reconcile the fleet once and exit
```

## Operating it

- **Reconciling.** The control plane reconciles every hub on a timer and on demand from the Admin tab. Reconciliation pushes the folder topology and access rules to each peer; it never copies files.
- **Adding a hub.** The Admin tab's **Add Mesh Hub** form takes a name, the peer's API URL, and its API key. The form validates both before writing. The peer-to-peer address is derived from the API host unless you set it in `fleet.yaml`.
- **Isolating a hub.** Disconnect marks a hub as disabled: reconciliation skips it, it is shown as disconnected, and its device is withdrawn from the mesh. Its entry in `fleet.yaml` is kept, so reconnecting is one click.
- **Share links.** A share link is a keyed MAC over the server secret and the target path. It grants access to that path and nothing else, and it cannot be forged without the secret.
- **Health.** `thrall --check` is the unit's health probe. The portal also reports fleet state at `/status` as JSON.

## Repository layout

```text
thrall/
├── AGENTS.md                  # Canonical House Laws (16-256 line Page Rule, zero deps)
├── Cargo.toml                 # Workspace root (release LTO, panic=abort, strip=true)
├── Containerfile              # Multi-stage Fedora-minimal control-plane OCI build
├── README.md                  # This file
├── assets/
│   ├── default-fleet.yaml     # Fleet configuration template
│   ├── login.js               # Portal client: SPA router, modals, upload engine
│   ├── portal.css             # Portal styling
│   ├── thrall-bg.jpg          # Primary Hive Thrall background
│   ├── thrall-bg-1..4.jpg     # 4 asymmetrical off-center Hive Thrall horror backgrounds
│   └── icons/                 # SVG + 16x16..512x512 PNG + multi-res ICO application icons
├── docs/                      # GitHub Pages landing site (https://ubermetroid.github.io/thrall/)
├── quadlet/                   # Hardened Podman systemd Quadlet unit for the control plane
└── thrall-daemon/
    ├── Cargo.toml             # Zero external dependencies (Apache-2.0)
    ├── src/                   # Modular pages, strictly <= 256 lines each
    └── tests/                 # Page Rule, lint, and security regression gates
```

## Development

Every source file is a "page": one idea, between 16 and 256 lines, at most 8 files per directory, and no external crates. The full standard is in [`AGENTS.md`](AGENTS.md).

The verification ladder runs on every commit:

```bash
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic
cargo fmt --all -- --check
cargo build --release --workspace
```

## License

Apache-2.0. See [LICENSE](LICENSE).
