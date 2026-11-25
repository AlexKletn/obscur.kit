What a [`ActivateWindow`](super::ActivateWindow) request should focus (spec §6.3).

Spec §6.3 separates *window activation* from *element focus*. This enum is a
tightly-coupled part of [`ActivateWindow`](super::ActivateWindow): it distinguishes activating the
window alone from also moving keyboard focus to a specific element, so it is
colocated here rather than given its own module.
