Runs the loop until something happens, or until `timeout` elapses, and
returns everything that did.

`None` sleeps until woken — by the compositor, or by a
[`LayerWaker`] given at open time. A duration bounds the wait, which is
what an animating host wants: it needs a turn per frame whether or not
anything arrived.

# Errors

[`LayerShellError::Dispatch`] when the connection or the loop fails.
