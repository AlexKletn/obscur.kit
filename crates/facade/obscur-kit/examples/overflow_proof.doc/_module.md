**Proof: CSS `overflow` clips a box's subtree.**

Two 100×60 boxes sit side by side, each holding a child that extends BEYOND
its box (down and to the right). The left box is `overflow: hidden`; the right
box is `overflow: visible`. This drives the REAL scene coordinator
([`SceneCoordinator::compose`]) — the same code path the runtime uses — which
brackets a non-`visible` node's subtree with a `PushClip`/`PopClip` around its
border box. It then rasterizes the composed display list headless through the
software [`CpuRenderer`] (no GPU) and writes `target/overflow_proof.png`.

Before writing it machine-checks the pixels: (1) in the left box's overflow
region (where the child spills past the box) the pixel is the WHITE background
— the child was CLIPPED; (2) in the right box's mirror overflow region the
pixel is GREEN — the child OVERFLOWS; and (3) inside the left box the child's
green still shows within the clip.

Run it:
`cargo run -p obscur-kit --example overflow_proof --features render-cpu`
