Loads and decodes the **densest** DPI variant in `candidates` that resolves,
tagging the resulting [`SceneImage`] with that variant's density (spec §5.16
DPI variants).

`candidates` is the ladder from
[`dpi_variants`](obscur_resources::source::dpi_variants): densest first,
always ending at the authored path. Each is tried in order and the first that
loads *and* decodes wins, so a project shipping no `@2x` asset transparently
falls back to the base one — a missing variant is the normal case, not an
error. The returned image carries the winner's density, which is what makes it
lay out at its logical (rather than pixel) size downstream.

# Errors

When no candidate resolves, this reports the **last** candidate's error — the
authored path's. That is the honest diagnostic: "`logo@2x.png` not found" would
name a file the author never wrote.
