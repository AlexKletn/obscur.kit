§2.6's routing chain, reached from an application: links 1 and 3, by name.

`UiRuntime` implements the whole chain — *focused node → focus scope →
ancestors → window → application fallback* — but the four positional links are
keyed by a `NodeId`, and a `NodeId` does not exist until a frame has built the
tree. An application configuring itself before the loop therefore had exactly
one link available (the application fallback), and the observable costs were one
flat command namespace, a symbol that could not mean two things in two places,
and handlers that could not sit beside the markup they serve.

What closes it is not a new routing mechanism but an *identifier that exists
before the tree does*: §7.5's stable `test-id` / `automation-id`, written on the
markup itself and resolved against the live tree after every commit. So these
cases go the whole way an application does — author a `test-id`, name it in a
registration, click the element, watch the right handler run — using only
`obscur_kit`'s public API.

The host contract is the pair of calls in [`Host::frame`]: run the frame, then
`AppControl::apply`. That is exactly what `App`'s window runner does, and it is
why these cases can prove the standalone entry point's behaviour without a
display (the windowed `App` needs `window-winit` + `render-vello`, so a test that
constructed one would be silently skipped in the default feature set).
