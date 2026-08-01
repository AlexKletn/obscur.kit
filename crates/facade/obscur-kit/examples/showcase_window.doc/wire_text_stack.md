Wires the full multi-font fallback stack onto the runtime and the Vello
renderer (identical to `windowed_counter.rs`): the primary under `sans-serif`
(id 0) plus a best-effort CJK fallback under the same family (id 1), so
resolution is by `cmap` COVERAGE, not name. The renderer receives the same
id↔bytes assignment so scene font handles line up with rasterized outlines.
