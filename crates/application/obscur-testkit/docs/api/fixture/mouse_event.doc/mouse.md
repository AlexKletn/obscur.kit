A trusted mouse event of `kind` at logical `(x, y)`.

These tests model a **real user** working the pointer — pressing a button,
dragging a selection, focusing a field. A real gesture is a trusted one, and only
a trusted gesture runs a click's gated default action (spec §2.3 step 4, enforced
in `handle_pointer`), so the event is marked as what it represents. A test that
wants the *untrusted* synthetic path — a headless or DevTools inject — must build
its own event and leave the trust bit off.
