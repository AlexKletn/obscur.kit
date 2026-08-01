Where an authorized synthetic input event is delivered. The endpoint never
touches the runtime itself (spec §7.7): the host owns the queue behind
this sink and applies what it drains (host-provides-device).

[`disconnected`](InjectionSink::disconnected) — the default — is a host
that cannot receive injected input, and every `Inject` is refused while it
is. That is the §7-C fix: this used to be a `Vec<InjectedEvent>` the
endpoint owned, which meant the events were unreachable from the *thread*
that owns the runtime, so they were acknowledged and then dropped.
