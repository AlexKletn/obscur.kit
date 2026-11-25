The reason `fonts` has no ceiling and no eviction, written as a test instead of
as a paragraph nobody has to believe.

`font_for` falls back to handle `0` when a run's own handle is not registered.
That is right for its intended case — a single-font host that only ever called
`set_font` — and it is exactly wrong for a face that WAS registered and then
dropped: glyph ids are face-local, so a Devanagari run replayed through the
primary Latin face does not go missing, it draws other letters.

Add eviction to this table and that is what eviction looks like on screen. It
cannot be undone by re-registering either, because the only path that delivers a
discovered face — `InstalledFonts::take_discovered` — drains its queue, so a
face is offered once and never again.

So this test is not a guard on today's behaviour so much as a price tag on a
change somebody will be tempted to make. If eviction ever does arrive it must
arrive together with a re-supply path and with `font_for` distinguishing "never
registered" from "dropped", and this test is where that shows up.
