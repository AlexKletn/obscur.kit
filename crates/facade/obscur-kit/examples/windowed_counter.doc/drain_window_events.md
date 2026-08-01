Drains the **port's** domain window events and acts on them (spec §6.1).

This is the §6.1 lifecycle path: the backend's handler converted the native
events, and the host reads them as backend-agnostic
[`DomainWindowEvent`]s — the same values a WASM or headless backend would
report — rather than matching on winit variants.

- `Resized` reconfigures the swapchain and the layout viewport.
- `CloseRequested` is a *request*: §6.1 leaves the decision to the
  application, so the host answers it by asking the port to close the
  window and then leaving the loop.
