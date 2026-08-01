A motion value sampled in the **pre-layout** phase (spec §4.17).

The motion engine produces presentation values in two phases keyed to the
layout pass. A `PreLayoutSample` carries a *layout-affecting* animated value —
one that changes geometry, such as an animated width, height or margin — that
must be resolved **before** the layout pass runs, so the layout engine reads
the animated input while measuring and arranging (spec §4.17 "before-layout
geometry"). Contrast [`PostLayoutSample`](super::PostLayoutSample), which
carries presentation-only values applied after layout.

The sample records which [`Timeline`](crate::timeline::Timeline) produced it,
the timeline [`progress`](Self::progress) at the moment of sampling, and the
resolved [`MotionValue`](Self::value). It is a plain immutable record: the
timeline is advanced elsewhere and each frame yields a fresh sample.
