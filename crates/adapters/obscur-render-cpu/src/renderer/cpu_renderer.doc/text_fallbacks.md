The [`SceneEffect::Text`] fallbacks this backend will take drawing `list`:
one per font handle whose runs will rasterize as box-coverage cells instead
of real glyph outlines (spec §5.2).

The predicate is [`draw_text_run`]'s own resolution rule, restated: an exact
registry hit whose bytes `skrifa` accepts. It has to be — a scan that
disagrees with the paint path reports a fallback that did not happen, or
stays quiet about one that did, which is the failure this whole diagnostic
exists to end.
