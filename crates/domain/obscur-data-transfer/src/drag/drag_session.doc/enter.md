Marks the pointer entering `target` and negotiates the effect (spec
§6.21).

If `target` accepts the dragged payload, the session records it as the
current target, sets the effect to the target's negotiated choice for
`preferred`, and enters phase [`Over`](DragPhase::Over). If the target
rejects the payload, the session clears the current target and effect and
returns to phase [`Started`](DragPhase::Started).

A no-op once the session is terminal.
