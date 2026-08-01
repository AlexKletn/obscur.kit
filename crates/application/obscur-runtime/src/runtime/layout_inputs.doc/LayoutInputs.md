The **non-style** inputs the retained boxes were arranged under.

Layout does not read the computed styles alone: it also reads the viewport,
the sticky scroll offset, and the focused editable's live text — a keystroke
grows a textarea's box without changing any computed style, which is exactly
why a multiline edit schedules a full [`FrameReason::Input`] frame. The
incremental classification in [`LayoutClassifier`](super::LayoutClassifier)
may only reuse the retained boxes — or the retained measures — while **all**
of these are unchanged.

[`FrameReason::Input`]: crate::frame::FrameReason::Input
