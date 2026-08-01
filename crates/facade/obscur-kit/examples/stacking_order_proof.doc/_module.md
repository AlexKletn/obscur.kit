**Proof: CSS 2.1 §9.9 seven-layer paint order (catalog bug B2).**

Two independent demonstrations sit side by side, driven through the REAL scene
coordinator ([`SceneCoordinator::compose`]) — the same code path the runtime
uses — then rasterized headless through the software [`CpuRenderer`] (no GPU) to
`target/stacking_order_proof.png`.

**Left — the lift (§9.9 layer 6 above layers 1–5).** A positioned `z:auto` box
`B` (BLUE) is the FIRST child; a non-positioned in-flow block `A` (GREEN) is the
SECOND child and overlaps it. Per §9.9 a positioned `z:auto`/`0` box paints
ABOVE non-positioned siblings regardless of tree order, so `B` must show ON TOP
of `A` in the overlap. The old flat `(z, tree_index)` sort painted `A` (later in
the tree) on top — the bug.

**Right — negative z below non-positioned (§9.9 layer 2).** A non-positioned
block `S` (YELLOW) is a child; a positioned box `N` (RED) with `z-index: -2`
comes LATER in the tree and overlaps it. A negative-z positioned box paints
UNDER non-positioned content, so `S` must show ON TOP of `N` even though `N`
comes later in document order.

Before writing the PNG it machine-checks the pixels at each overlap.

Run it:
`cargo run -p obscur-kit --example stacking_order_proof --features render-cpu`
