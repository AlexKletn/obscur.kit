Renders one deterministic frame that **matches authored stylesheets**
against the reconciled tree, and returns its [`FrameResult`].

This is the headless equivalent of a frame: it force-renders the runtime
([`UiRuntime::render_now`]) over the host-owned state, running
`View -> reconcile -> match -> style -> motion overlay -> layout -> scene`
synchronously, and advances the frame counter. No OS or GPU is involved.

The frame reflects the clock: any motion registered on the runtime is
overlaid at the progress [`advance_time`](Self::advance_time) has driven
it to. Two calls with the same view, sheets, and clock reading produce the
same display list.

`sheets` must be ordered by ascending cascade origin (system-default
first, inline last). An empty `sheets` slice matches nothing.
