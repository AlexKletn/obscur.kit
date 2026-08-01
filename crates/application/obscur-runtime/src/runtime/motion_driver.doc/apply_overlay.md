Overlays every animation binding's sampled value onto `computed` before
layout and paint (spec §4.17 style → motion → paint).

For each binding: if its timeline is still active in the scheduler the
track is sampled at that timeline's progress; if the timeline has finished
(and was pruned) the track is sampled at its endpoint `t == 1.0`, so the
settled value keeps being applied. The sampled scalar is lowered to a
[`PropertyValue`] per the binding's [`AnimatedKind`], wrapped as a
[`ComputedValue`], and written onto the target node's [`ComputedStyle`]. A
node with no computed entry (e.g. `display: none`) is skipped, and a
non-scalar sample or an empty track contributes nothing.

[`PropertyValue`]: obscur_style::property::PropertyValue
[`ComputedStyle`]: obscur_style::computed::ComputedStyle
