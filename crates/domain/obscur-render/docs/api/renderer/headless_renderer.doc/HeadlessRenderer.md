A deterministic, no-GPU reference [`Renderer`] — the software/test backend of
spec §5.2.

`HeadlessRenderer` presents every submitted frame by echoing its scene
generation, so a test can assert exactly which generation "reached the
screen" without a real device. It counts the frames it has presented and holds
no GPU or OS handle.

It rasterizes **nothing** — there is no pixel buffer here at all — so it
reports [`SupportLevel::Unavailable`] for every effect, the same honest answer
the other null objects in this workspace give. (It previously claimed
`Emulated` for everything on the grounds that "a software rasterizer emulates
all effects"; this one is not a rasterizer.) For a real software backend that
draws pixels, see `obscur-render-cpu`.
