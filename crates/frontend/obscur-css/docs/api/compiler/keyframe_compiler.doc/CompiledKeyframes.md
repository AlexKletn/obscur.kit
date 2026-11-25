A compiled `@keyframes` rule: an interned name and its per-property tracks.

`tracks` holds one entry per animated [`PropertyId`], in first-seen order,
each paired with a [`KeyframeTrack`] whose keyframes are sorted by offset.
