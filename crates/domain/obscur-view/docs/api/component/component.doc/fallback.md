The fallback UI this component's boundary shows in place of its subtree
once a contained failure is out of retries (spec §1.7). Default: `None`
— the subtree is replaced by nothing, so a **contained failure that is
out of retries** never leaves the last good output on screen pretending
to be current.

Two paths do still leave it standing, and neither is a fallback's to
answer: a retry, for the frames its budget buys, and a propagated
failure that reaches no enabled boundary. Both are reported. See
`render_components`' "Where the previous output still stands".

Read once, when the runtime creates the instance, and never asked for
again: it must not depend on the render that failed, which is exactly the
state a fallback cannot trust.
