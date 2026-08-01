The [`SceneEffect::Text`] fallbacks this backend will take drawing `list`:
one per font handle whose runs will be skipped for want of a font
(spec §5.2).

This backend's restriction on `Text` is harsher than the CPU rasterizer's —
there is no box-coverage stand-in here, the run is simply not drawn — which
is why naming it matters most: nothing appears, so there is no artifact for a
host to notice.

The predicate is [`SceneConverter::can_draw_text`], i.e. `font_for`'s registry
lookup, because that is the only gate the draw path applies. It is therefore
**narrower than the CPU backend's**, which also requires the bytes to parse:
registering unparseable bytes here satisfies this scan and raises nothing,
and Vello's own glyph pipeline then **panics** rather than drawing nothing
(`Scene::draw_glyphs` unwraps its `FontRef`). Reporting it would mean parsing
fonts this backend never parses, so it is left un-diagnosed rather than
guessed at — but the failure is a crash, not a blank run, and a host that
registers untrusted bytes owns that.
