The result of routing an event through its propagation path (spec §4.7).

Summarizes what happened while an event traveled its capture → target →
bubble path: whether any node [`handled`](Self::is_handled) it, whether the
host's default action was [`prevented`](Self::is_default_prevented), and, if
propagation was cut short, the [`node`](Self::stopped_at) at which it stopped.
