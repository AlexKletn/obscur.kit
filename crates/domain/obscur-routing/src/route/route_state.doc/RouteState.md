Opaque, application-defined state carried alongside a navigation entry
(spec §6.13 typed route state).

Applications frequently attach transient state to a history entry — scroll
position, unsubmitted form data, a selection — that should be restored on
back/forward navigation. Because the concrete shape is application-defined,
the toolkit models it as an opaque carrier rather than interpreting it:

- [`kind`](RouteState::kind) is an application-chosen type tag (for example
  `"scroll"`) that lets the application decide how to decode the payload.
- [`payload`](RouteState::payload) is the application's serialized state
  blob.

The toolkit never inspects the payload; serialization and navigation policy
are application concerns outside the toolkit's scope.
