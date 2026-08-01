The typed event that travels §2.3's propagation path and is handed to every
handler along it (spec §2.3, §4.7).

§2.3 enumerates the API an event must expose, and this is that list:

> Event API includes:
>
> - `target`;
> - `current_target`;
> - `stop_propagation`;
> - `stop_immediate_propagation`;
> - `prevent_default`;
> - trusted/untrusted origin.

[`target`](Self::target) is the node the event is *aimed* at — the hit-test
result for a pointer, the focused node for a key — and does not change while
the event propagates. [`current_target`](Self::current_target) is the node
whose handler is running *right now* and changes at every step; the pair is
the whole reason a bubbled handler can tell "what was clicked" from "what I am
attached to". [`propagation`](Self::propagation) holds the three steering
flags, [`trust`](Self::trust) is §2.3's trusted/untrusted origin, and
[`payload`](Self::payload) is §2.4's device state.

# What this type used to be

Until this change `UiEvent` carried only `origin`, `target`, `phase`,
`propagation` and a timestamp, and — more to the point — **nothing
constructed one**: `UiEvent` and `EventDispatcher` appeared outside this crate
in exactly two places, a prelude re-export and a spec test. The runtime
resolved input to a node and then ran a `FnMut()` that was told nothing at
all, so `current_target`, the pointer's position, the key, and the trust flag
were unreachable from application code even though every one of them was
already modeled here. The fields added below (`current_target`, `trust`,
`payload`) are the ones a handler could not have obtained; the rest of the gap
was delivery, and lives in `obscur_runtime`.

# A target is optional, and that is not a placeholder

[`target`](Self::target) is `Option<NodeId>` because §2.6's routing chain ends
in two links that are **not tree positions** — *"4. window; 5. application
fallback"*. A command dispatched with nothing focused reaches those links and
only those, and it genuinely has no target node; reporting some sentinel node
would be a lie a handler could act on.

# Not `Eq`

The pointer payload carries `f32` pressure, position, tilt and rotation, so
this type is `PartialEq` only. It was `Eq` while it carried no payload.

The timestamp is a *logical* value supplied by the caller (for example, from
the platform event or a test harness); this crate never reads a wall clock,
so routing stays pure and deterministic.
