# Multi-stage hardened Podman Containerfile for THRALL
# Bundles:
#   - /bin/thrall: Zero-dependency static Rust LDAP + Syncthing mesh controller
#   - /bin/syncthing: Official static Syncthing binary (when container also acts as a local peer node)
#   - openssh-client & curl: Zero-shell SSH transport for remote fleet management

FROM docker.io/rust:1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml ./
COPY assets ./assets
COPY Containerfile ./
COPY quadlet ./quadlet
COPY thrall-daemon ./thrall-daemon
RUN cargo build --release --workspace && strip target/release/thrall

FROM docker.io/syncthing/syncthing:latest AS syncthing-bin

FROM docker.io/library/alpine:3.21
RUN apk add --no-cache openssh-client curl ca-certificates tzdata \
    && adduser -D -u 1000 -g 1000 thrall \
    && mkdir -p /config /users /shared/.stfolder \
    && chown -R 1000:1000 /config /users /shared

COPY --from=syncthing-bin /bin/syncthing /bin/syncthing
COPY --from=builder /src/target/release/thrall /bin/thrall
COPY assets/default-fleet.yaml /config/fleet.yaml

EXPOSE 8080/tcp 8384/tcp 22000/tcp 22000/udp
VOLUME ["/config", "/users", "/shared"]

USER 1000:1000
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["/bin/thrall", "--check"]

ENTRYPOINT ["/bin/thrall"]
CMD ["--config", "/config/fleet.yaml"]
