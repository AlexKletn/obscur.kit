The three ways a frame is run: the scheduled one, the scheduled *reactive* one,
and the unconditional one.

All three end in the same call — the pipeline in `frame_pipeline.rs` — and they
differ in exactly two things: whether the scheduler is consulted, and where the
desired [`View`](obscur_view::view::View) comes from.
[`run_due_frame`](UiRuntime::run_due_frame) is handed one;
[`run_due_frame_reactive`](UiRuntime::run_due_frame_reactive) builds one from the
retained root builder, and hands the builder in *lazily* so it runs after the
frame's input-dispatch phase rather than before it;
[`render_now`](UiRuntime::render_now) ignores the scheduler entirely.

The two scheduled entries share a preamble that is not shared code by accident:
§3.6's starvation check runs first, then one request is pulled, then the rest of
the scheduling window is coalesced into it (§3.4 step 2, §3.5). Written in both
because the reactive path's early `None` returns — no builder installed, nothing
due — must not have side effects, and the order in which they are taken is what
guarantees it.
