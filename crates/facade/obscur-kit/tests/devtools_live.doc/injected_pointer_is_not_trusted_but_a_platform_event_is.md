The security property §2.3's trusted flag exists for, asserted against the
real inject path rather than a mock: `apply_injected` — the function §7.6's
agent loop reaches — must produce untrusted input, while the same click made
the way a platform adapter makes it is trusted.

This is deliberately a *contrast*, not an existence check: asserting only that
the field is readable would pass with `trust` wired to a constant. Here the
two events differ in exactly one respect — who built them — and the runtime
must report a different verdict for each.

The trusted half of the pair against the *real* producer (the winit converter,
which is not reachable from this crate) is
`a_converted_os_event_is_trusted` in `obscur-window-winit`'s
`conversion::pointer_converter`. The two fail disjointly: adding
`.as_trusted()` to `apply_injected` fails only this test; deleting it from the
converter fails only that one.
