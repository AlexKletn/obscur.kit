Sets the device-pixel scale (HiDPI factor × user zoom) the composed scene
renders at, clamped to a sane minimum of `0.1` (a non-positive or
non-finite request is ignored in favour of the clamp) (spec §5.2).

The scale wraps the composed display list in a single root
`PushTransform(scale(s, s))` … `PopTransform`, so a host that lays out in
**logical** CSS pixels (viewport = physical extent / `s`) still rasterizes
crisply at physical resolution. Pointer coordinates are divided by `s` in
[`handle_pointer`](Self::handle_pointer) so hit-testing stays in logical
space. Setting `1.0` restores the byte-identical (untransformed) path.

Moving to a display of a different density re-resolves images against the
new [DPI-variant](obscur_resources::source::dpi_variants) ladder: an app
dragged from a 1x to a 2x screen picks up its `@2x` assets rather than
keeping the ones it happened to load first (spec §5.16). Only a change that
selects *different files* does this — see
[`invalidate_dpi_variants`](Self::invalidate_dpi_variants).

Does not itself schedule a frame; a host typically pairs this with
[`set_viewport`](Self::set_viewport) and
[`request_frame`](Self::request_frame)`(`[`FrameReason::WindowChange`]`)`.
