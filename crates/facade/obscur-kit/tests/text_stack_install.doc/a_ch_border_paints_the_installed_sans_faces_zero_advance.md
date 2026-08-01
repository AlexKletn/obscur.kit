The installed face's `0` advance must reach **paint**, not just layout.

# Why this exists next to the test above, which looks identical

`ch_resolves_to_the_installed_faces_measured_zero_advance` reads `item_width`
off the **layout** boxes, and layout has always been able to measure `ch`
itself (`ch_width_for`). So it passes whether or not the computed-value step
resolves `ch` — verified by neutering that step, which leaves it green. It
pins the *font*, and cannot pin the *seam*.

Paint is the half that could not measure: it reads the computed property
through `Length::as_px`, which honours absolute `px` only, so a `ch` that
survives the fold reaches it as `0`. Asserting a painted border edge is
therefore the assertion layout's fallback cannot rescue.

The sans face is load-bearing here for the reason the module header gives:
Noto Sans Mono's `0` is `600/1000` and `MockTextShaper::ADVANCE_RATIO` is
`0.6` — equal at every size, so a mono assertion passes with the whole install
reverted. Noto Sans's `572/1000` discriminates all three candidate answers:
9.152 measured, 9.6 mocked, 8.0 substituted.
