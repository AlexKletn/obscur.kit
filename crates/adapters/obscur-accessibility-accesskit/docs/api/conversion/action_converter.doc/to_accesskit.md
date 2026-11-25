Maps a domain [`SemanticAction`] to the AccessKit [`Action`] a node
should advertise, or [`None`] when AccessKit has no equivalent.

[`SemanticAction::Activate`] maps to [`Action::Click`]. The clipboard
actions ([`Copy`](SemanticAction::Copy), [`Paste`](SemanticAction::Paste),
[`Cut`](SemanticAction::Cut)), [`Select`](SemanticAction::Select), and
[`Dismiss`](SemanticAction::Dismiss) have no AccessKit action and return
[`None`].
