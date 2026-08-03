Builds the press/release for **any** of the five modelled buttons.

It used to build one: the caller was guarded to `MouseButton::Left` and this
hardcoded `PointerButtons::PRIMARY`, so a right click reached nothing and there
was no context menu because there was no right click. The mapping had been
written and tested in `PointerConverter::button` the whole time, with no caller.

It goes through [`PointerConverter`]'s `pressed`/`released` builders rather
than `PointerEvent::new`, because those are the ones that mark the event
**trusted** (§2.3). This is the platform-adapter path — a real user gesture —
and a gated default action (the click → command dispatch) only runs for a
trusted event, so building the event by hand here silently disabled every
click in every `App`.
