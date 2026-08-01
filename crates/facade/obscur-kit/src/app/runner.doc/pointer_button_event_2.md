Builds the press/release for a left-button change.

It goes through [`PointerConverter`]'s `pressed`/`released` builders rather
than `PointerEvent::new`, because those are the ones that mark the event
**trusted** (§2.3). This is the platform-adapter path — a real user gesture —
and a gated default action (the click → command dispatch) only runs for a
trusted event, so building the event by hand here silently disabled every
click in every `App`.
