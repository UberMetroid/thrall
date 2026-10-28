# Multi-stage hardened Fedora-minimal Podman Containerfile for THRALL
# Bundles:
#   - /bin/thrall: Zero-dependency Rust LDAP + Syncthing mesh controller (stripped, glibc)
#   - openssh-clients & curl: Zero-shell SSH transport for remote fleet management
# The Syncthing peer node is NOT bundled here: it runs from the dedicated
# thrall-syncthing image (Containerfile.syncthing) via quadlet/syncthing.container.
# Fedora-native supply chain: signed packages from Fedora repos only - no Alpine/BusyBox, no Guix.

ARG FEDORA_MAJOR=44
FROM registry.fedoraproject.org/fedora-minimal:${FEDORA_MAJOR} AS builder
RUN microdnf install -y rust cargo gcc
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY thrall-daemon ./thrall-daemon
COPY assets ./assets
RUN cargo build --release --workspace

FROM registry.fedoraproject.org/fedora-minimal:${FEDORA_MAJOR}
# Strip the package managers from the runtime image (smaller attack surface).
# /usr/bin/microdnf is shipped by the dnf5 package itself, and dnf5 is a
# protected package - so empty the protection list for this one transaction.
RUN microdnf install -y openssh-clients curl ca-certificates tzdata \
    && groupadd -g 1000 thrall \
    && useradd -u 1000 -g 1000 -M -d /config -s /sbin/nologin thrall \
    && mkdir -p /config /users /shared/.stfolder \
    && chown -R 1000:1000 /config /users /shared \
    && microdnf clean all \
    && microdnf remove -y --setopt=protected_packages= dnf5 libdnf5 libdnf5-cli \
    && rm -rf /var/cache/* /var/log/* /tmp/*

COPY --from=builder /src/target/release/thrall /bin/thrall
COPY assets/default-fleet.yaml /config/fleet.yaml

EXPOSE 8080/tcp
VOLUME ["/config", "/users", "/shared"]

USER 1000:1000
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["/bin/thrall", "--check"]

ENTRYPOINT ["/bin/thrall"]
CMD ["--config", "/config/fleet.yaml"]
