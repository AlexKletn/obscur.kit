[`CssPipeline`]: the host-side CSS path — parse (cached) → compile → install
onto a [`UiRuntime`](crate::runtime::UiRuntime).

The CSS frontend and the runtime are deliberately unaware of each other: the
frontend emits `obscur-css` types, the runtime accepts only domain types
(`obscur-style`'s [`Stylesheet`] and `obscur-runtime`'s [`CssMotionRules`]).
Joining them is a fixed piece of plumbing that every CSS-rendering host needs
and none should hand-write — so it lives here, next to
[`install_vector_resources`](super::KitBootstrap::install_vector_resources),
rather than in each application.

Two things are easy to get wrong by hand and are why this type exists:

- **Motion is dropped.** A host that installs only `compiled.matchable` gets
  a runtime that never sees the sheet's `transition`/`@keyframes`, so CSS
  authored motion silently does nothing (§4.17).
- **Sources are re-parsed.** A reload recompiles the whole sheet set, but only
  the edited source's *text* changed; re-parsing the rest is exactly the work
  §4.12's [`StylesheetCache`] exists to skip.
