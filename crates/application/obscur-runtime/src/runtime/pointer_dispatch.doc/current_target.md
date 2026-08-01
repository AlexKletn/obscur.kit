§2.3's `current_target`: the node whose binding resolved
[`command`](Self::command) — the element the handler is *attached to*, as
against [`target`](Self::target), the element the event *hit*.

Equal to `target` when the hit node carries the binding itself, and an
**ancestor** of `target` when the event bubbled to find one. `None`
exactly when `command` is `None`: no binding resolved, so no handler ran,
so nothing is a current target.
