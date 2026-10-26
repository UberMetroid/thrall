;; THRALL (`st-mesh-mgr`) — Declarative GNU Guix System & Container Manifest
;; Build a hermetic OCI/Docker image for Podman/Quadlet:
;;   guix pack -f docker -S /bin=bin -S /etc=etc -m guix.scm
;; Or spawn an isolated Guix container directly:
;;   guix shell --container --network --share=/srv/thrall-exp/thrall-config=/config \
;;     --share=/srv/thrall-exp/users=/users --share=/srv/thrall-exp/shared=/shared \
;;     -m guix.scm -- thrall --config /config/fleet.yaml

(use-modules (gnu packages certs)
             (gnu packages curl)
             (gnu packages ssh)
             (gnu packages syncthing)
             (guix build-system cargo)
             (guix gexp)
             ((guix licenses) #:prefix license:)
             (guix packages))

(define-public thrall-daemon
  (package
    (name "thrall")
    (version "0.2.0")
    (source (local-file "." "thrall-checkout" #:recursive? #t))
    (build-system cargo-build-system)
    (arguments
     `(#:install-source? #f
       #:phases
       (modify-phases %standard-phases
         (add-after 'install 'install-default-config
           (lambda* (#:key outputs #:allow-other-keys)
             (let* ((out (assoc-ref outputs "out"))
                    (bin (string-append out "/bin"))
                    (etc (string-append out "/etc/thrall")))
               (mkdir-p etc)
               (copy-file "assets/default-fleet.yaml"
                          (string-append etc "/fleet.yaml"))
               (symlink (string-append bin "/thrall")
                        (string-append bin "/st-mesh-mgr"))
               #t))))))
    (inputs (list curl nss-certs openssh-sans-x syncthing))
    (home-page "https://ubermetroid.github.io/thrall/")
    (synopsis "Zero-dependency Rust Multi-Server Syncthing LDAP Control Plane")
    (description
     "THRALL (Tool for Hierarchical Replication And LDAP Linking) is a pure-Rust
multi-server Syncthing mesh controller, LDAPv3 authenticator, and web file explorer.")
    (license license:asl2.0)))

(packages->manifest
 (list thrall-daemon
       curl
       nss-certs
       openssh-sans-x
       syncthing))
