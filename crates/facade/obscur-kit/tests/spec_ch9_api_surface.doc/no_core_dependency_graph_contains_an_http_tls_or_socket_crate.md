§9.1 — "- arbitrary networking in the core;" and §1.3 "Unused backends and
optional integrations must not be linked."

NG-NETDEPS-01. (The per-crate resolved dependency graph is not reachable from
a test without shelling out to cargo — which would contend for the build lock
— so this asserts the two observable proxies: no HTTP/TLS/socket crate is in
the workspace lockfile at all, and the umbrella's default feature set names no
GPU or windowing backend.)
