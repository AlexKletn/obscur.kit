The element names the UA sheet styles that reach the tree only through
hand-written markup.

This is the guard's only declared vocabulary, and it is deliberately three
names long. `a` and `fieldset` are HTML element names with no counterpart in
the widget or primitive tier; they reach the tree through hand-written markup,
where `name_binding::local_name_for` makes every element tag its own local
name. `the_markup_only_vocabulary_really_reaches_the_tree` proves each entry,
so an addition here has to be an element that renders — but it is still a
declaration, which is why it is short and why adding to it should be argued.

`canvas` is here for a different and less comfortable reason, and it is
recorded rather than hidden: a primitive *does* build one — `Canvas::new(…)`,
and every `<progress>` and `<slider>` contains one — but `Canvas::into_view_node`
contributes no `LOCAL_NAME`, so what it builds arrives in the tree with no CSS
type at all. That is exactly defect 2 of the audit this file documents, the one
fixed for `img` and never for `canvas`, and the consequence is that the sheet's
`canvas { … }` rule reaches the authored tag and not the primitive. Naming the
primitive is a one-line change in `crates/components/obscur-primitives/src/canvas.rs`;
the day it happens, that rule starts reaching the canvas *inside* every drawn
control, which is why the sheet sizes those children after it and why
`the_pointer_track_and_the_painters_track_are_one_box` measures the result.
