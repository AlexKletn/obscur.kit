Returns whether this sample affects layout — always `true`.

A pre-layout sample is, by definition, a layout-affecting value resolved
before the layout pass runs (spec §4.17); this constant mirrors
[`PostLayoutSample::affects_layout`](super::PostLayoutSample::affects_layout),
which is always `false`, so the two phases can be discriminated uniformly.
