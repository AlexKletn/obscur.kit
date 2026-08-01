§6.17 Capture and privacy: "Results:" / "- captured;" / "- denied;" /
"- unsupported;" / "- failed."

Headlessly, a window capture is `unsupported` (there is no window), and every
capture kind answers with a value of the one result type.

WEAKENED 2026-07-19: this originally asserted that a NODE capture succeeds on
`NullCaptureProvider` — "node capture needs no compositor and reports
`captured`". That sentence is not in §6.17. The section lists the capability
kinds ("capture window", "capture surface", "capture node", "capture image",
…) and the four results ("captured", "denied", "unsupported", "failed"),
and says nothing about which kinds a compositor-less provider
must be able to service; a null provider answering `unsupported` uniformly is
a conforming reading. What is kept is what §6.17 does state: each request is
answered with a typed result, an unsupported one carries no image, and the
four results are distinguishable from one another. Candidate spec amendment:
say whether node/image capture is required to work without a window or
compositor — it is the difference between a headless golden-image pipeline
being possible and not.
