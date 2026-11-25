The **host** end of the injection channel: a bounded, thread-shared queue of
synthetic input events an authorized DevTools client asked for (spec §7.6
agent loop, §7.7 host-provides-device).

# Why this exists at all

The endpoint used to hold the injected events in a private `Vec` and hand them
out through a `drain_injections(&mut self)`. Nothing ever called it, and the
reason is structural rather than an oversight: a host serves its endpoint on
its **own thread** (the endpoint owns the session — token, scopes, audit log —
and a `UiRuntime` is not `Send`), so the thread that owns the runtime does not
own the endpoint and could not reach the drain. Every `Inject` was answered
`Acknowledged` and dropped.

So the channel is inverted, exactly as the *outbound* half already is: the
host makes the queue, keeps it, and hands the endpoint a [`sink`](Self::sink).
The serving thread pushes; the host drains on its own thread and applies each
event to its runtime (`obscur_kit::devtools::apply_injected`). This is the
mirror image of [`EventQueue`](crate::transport::EventQueue), and for the same
reason: the endpoint decides *whether* it happens, the host decides *what it
does*, and neither reaches into the other's thread.

# What a sink proves, and what it does not

The endpoint refuses an `Inject` when it holds no live sink, so a host that
cannot receive injected input says so instead of acknowledging it — that is
the §7-C half this closes, and it is a fact about reachability, not a promise:
a [`sink`](Self::sink) holds a [`Weak`], so when the host drops its queue the
endpoint's sink goes dead and injection is refused again.

It does **not** prove the host drains. A host that builds a queue, hands out a
sink, keeps it alive and never calls [`drain`](Self::drain) has re-created
drop-on-the-floor for itself, and no type here can see that. What is now
impossible is the state every host in this workspace was actually in: no
receiver at all, and an acknowledgement anyway.

# Bounded on purpose

[`MESSAGE_LIMIT`], the same ceiling §7.7 imposes elsewhere. A client that
outruns the host's frame rate **drops the oldest** event rather than growing
without bound, and [`dropped`](Self::dropped) counts the loss. Unbounded, a
paused host plus a chatty agent is a memory leak in the app being inspected.
