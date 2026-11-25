What a dispatched [`SyntheticInput`](crate::input::SyntheticInput) reports
through the runtime (spec §7.4).

The result carries the resolved [`target`](Self::target), its
[`propagation_path`](Self::propagation_path) from target to root, the
resolved [`command`](Self::command), and changes to
[`hover`](Self::hover_changed) and [`focus`](Self::focus_changed).
[`default_prevented`](Self::default_prevented) records a refusal from a raw
keyboard, pointer, or context-menu handler. The retained tree and layout boxes are
available for further assertions.
