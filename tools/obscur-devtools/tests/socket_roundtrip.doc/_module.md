End-to-end: the DevTools CLI client drives the demo host over a **real**
loopback socket (spec §7.6/§7.7). One process, two threads, one kernel
socket — exercising handshake → authorize → inspect → typed reply through the
unchanged endpoint and the same `WireCodec` framing a separate process uses.
