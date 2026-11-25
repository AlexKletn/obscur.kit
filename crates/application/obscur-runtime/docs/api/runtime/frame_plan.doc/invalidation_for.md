Maps a [`FrameReason`] to the global [`InvalidationReason`] the router
classifies for it, or `None` for an idle reason that recalculates nothing.

The structural reasons (`UiCommit`/`ExternalRender`/`ResourceCompletion`)
map to [`TreeChanged`](InvalidationReason::TreeChanged) — the deepest
non-environment cause — so they dirty every phase; `WindowChange` maps to
[`EnvironmentChanged`](InvalidationReason::EnvironmentChanged) so media
queries re-evaluate without a tree diff; `Input` maps to
[`PseudoStateChanged`](InvalidationReason::PseudoStateChanged) so a
hover/focus change restyles without reconciling; `Animation` maps to
[`StyleChanged`](InvalidationReason::StyleChanged) (conservative: an
animated value can feed layout). `DiagnosticsCapture` maps to `None` — the
frame only re-presents the retained scene.
