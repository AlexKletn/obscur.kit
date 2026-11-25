Two claims about the attachment load op, both of them about the same pass.

The first is correctness and is asserted: `Load` and `Clear` produce the SAME
pixels. The test dirties the target with opaque green, blits under `Clear`,
reads every texel back; then dirties it again, blits under `Load`, reads every
texel back; and compares the two readbacks whole. They are equal because the
triangle covers the attachment and the pipeline blends nothing, so whatever the
`Load` arm loaded is overwritten without ever being read. If that ever stops
being true — a scissor, a viewport, a blend state, a triangle that no longer
spans the target — this fails, and it fails on the readback rather than on
somebody's reading of the shader.

The second is cost and is only PRINTED. It times both ops at the size the op's
cost is proportional to (the tablet panel, 2944x1004 physical), fifty passes to
a submit so the number divided out is GPU work rather than submit latency, in
alternating blocks so a machine drifting under load drifts through both arms,
keeping each arm's best batch. A benchmark cannot be an assertion here: this
desktop's iGPU is shared with a compositor and a browser, and a threshold that
holds on an idle box is a flake on a loaded one. So it reports, and the decision
it fed is recorded in `LOAD_OP.md`, where a later run on other hardware can
contradict it in writing.

Ignored by default and fatal when asked for by name — see `gpu.md` and
`NO_ADAPTER.md` for why that pair is the honest shape for a GPU test here.
