Returns whether this sample affects layout — always `false`.

A post-layout sample is a presentation-only value applied after layout
(spec §4.17); it never changes geometry. Mirrors
[`PreLayoutSample::affects_layout`](super::PreLayoutSample::affects_layout),
which is always `true`.
