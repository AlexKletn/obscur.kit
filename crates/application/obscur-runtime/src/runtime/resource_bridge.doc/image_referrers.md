The node that first referenced each requested image URL, retained so a
resource load/decode failure can be attributed to the node that asked for
it (spec §1.6 "node/resource/window context"). Populated at request time
in the resolve passes — the frame thread is where a URL and its referring
node are known together — and read back by
[`report_failure`](Self::report_failure) when the load, which may be
deferred out of the frame phase, finally fails.

Swept alongside [`image_handles`](Self::image_handles) by
[`retain_referenced`](Self::retain_referenced): an attribution for a URL
nothing refers to and no cache holds can never be read again.
