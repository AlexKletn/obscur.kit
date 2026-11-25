The typed input payload carried by a [`UiEvent`](super::UiEvent) (spec §2.3,
§2.4).

§2.3 closes with a sentence that rules out the shape a payload is *usually*
smuggled in with: *"Events are typed. String-based method dispatch is not
used."* So the payload is an enum of the concrete device events the runtime
already models — not a property bag, not a `dyn Any`, and not a name a
handler looks up. A handler reads it by matching, and the compiler knows which
fields exist.

# Why the envelope does not simply *contain* the pointer fields

§2.4 lists nine facts a pointer event must carry (pointer ID, device type,
buttons, logical position and delta, pressure, tilt, rotation,
primary-pointer status, hover capability); a keyboard event carries none of
them and carries a physical/logical key pair instead. Flattening both into one
struct would give every handler a set of fields that are meaningless for the
event it actually received — which is the untyped shape §2.3 excludes. The
variants keep "which facts exist" a property of the event's *kind*.

# `Empty`

[`Empty`](Self::Empty) is not a device: it is the honest payload for a
command that no device produced — an accessibility action, a menu item, a
programmatic dispatch. Such a command still propagates and still reaches
handlers, so it still needs an envelope; it simply has no device state to
report, and a handler that matches on the variant sees that rather than
reading a zeroed position that never existed.
