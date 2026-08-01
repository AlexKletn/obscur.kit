**Proof: CSS 2.1 §8.3.1 vertical (block-axis) margin collapsing in block flow.**

Four columns, each a `display: block` container laid out by the REAL
[`FlowLayout`] algorithm (the `gap`-only stack now also applies and collapses
block margins). Each column stacks a blue block **A** over a green block **B**
with the container's light-grey showing through the space between them:

* **(a)** `gap: 0`, A `margin-bottom: 20`, B `margin-top: 30` → the margins
  collapse to `max(20, 30) = 30`, NOT `50`. B's top = A's bottom + 30.
* **(b)** `gap: 0`, A `margin-bottom: 20`, B `margin-top: −8` → mixed set sums
  to `20 + (−8) = 12`.
* **(c)** `gap: 0`, an empty zero-height spacer with `margin: 15 0` between A
  and B → it collapses *through*, so A→B is ONE 15px gap, not two.
* **(d)** control: margins `0`, `gap: 10` → unchanged 10px separation
  (byte-identical to the pre-collapsing gap-only stack).

It first self-asserts the child y-positions straight out of [`FlowLayout`]
(the load-bearing proof), then feeds those computed boxes into the REAL
[`SceneCoordinator`] + software [`CpuRenderer`] (no GPU) and machine-checks
the rasterized pixels before writing `target/margin_collapse_proof.png`.

Run it:
`cargo run -p obscur-kit --example margin_collapse_proof --features render-cpu`
