Raises one structured [`Diagnostic`] per node-attributed "invalid value"
style fault this frame's match produced (spec §1.6, §7.8, §4.13).

Each dropped, unparseable declaration value (`width: not-a-length`)
carries the same stable code and subsystem regardless of which property
or which node it came from — the fault class is one thing — while the
node context differs so a tool points at the exact element. The property
is kept out of the message so two faults on two properties read
identically; it is a typed record on the matched
[`StyleRule`](obscur_style::matching::StyleRule), not interpolated text.
The value was dropped and the rest of the block still applies, so the
recovery is [`RecoveryAction::Fallback`].
