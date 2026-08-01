Publishes a live runtime's frame and focus transitions onto a DevTools
[`EventQueue`] (spec §7.6 [`Topic::Frames`] / [`Topic::Focus`]).

The host calls [`publish`](Self::publish) once per frame; each call compares
the runtime's real state against what was last published and emits an event
only on a genuine change — so the stream reports transitions the runtime
actually made, and an idle frame produces nothing.

# What it reports, and what it does not

* [`Topic::Frames`] — one [`EventKind::FrameCommitted`] per advance of the
  runtime's [`commit_version`](crate::runtime::UiRuntime::commit_version),
  carrying that version as the id. A frame whose reconcile changed nothing
  does not advance the version, so it is not reported: the topic is
  *tree-commit* events, not present/vsync events.
* [`Topic::Focus`] — one [`EventKind::FocusChanged`] per change of
  [`focused`](crate::runtime::UiRuntime::focused).
* [`Topic::Lifecycle`] — **nothing**. The runtime exposes no lifecycle
  transition stream to derive it from, so a `Lifecycle` subscriber on a host
  that publishes only through this type is granted a subscription that stays
  silent. That gap is open.
