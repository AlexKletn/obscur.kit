Reports a component failure on the diagnostics channel (spec §1.6).

The boundary already built the record; this anchors it to the `node` it
concerns and stamps the `recovery` action that was actually taken, which
are the two §1.6 fields only the runtime knows.
