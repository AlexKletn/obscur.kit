A system notification (spec §6.12 — notifications).

A notification is a pure value describing what the app wants shown: a
[`title`](Self::title) and [`body`](Self::body), an optional app-icon
[`badge`](Self::badge) count, optional determinate [`progress`](Self::progress),
a set of actionable [`actions`](Self::actions), and whether it should
[request attention](Self::request_attention) (bounce/flash). Whether any of
this is honored is the host's decision — permission and presentation policy are
explicit and live behind the
[`NotificationProvider`](crate::notification::NotificationProvider) port
(spec §6.12).

Build one with [`new`](Self::new) and layer on optional facets with the fluent
`with_*` / `requesting_attention` methods.
