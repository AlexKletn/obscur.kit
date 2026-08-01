Converts between the domain [`SemanticAction`] set and AccessKit [`Action`]
requests in both directions (spec §7.1 actions, §2.7).

The converter is stateless. [`to_accesskit`](Self::to_accesskit) advertises
which actions a node supports when a tree is pushed; its inverse,
[`from_accesskit`](Self::from_accesskit), turns an inbound platform
[`accesskit::ActionRequest`]'s action back into a domain action so it can
enter the same command/event model as ordinary input (spec §7.1).

Not every action has a counterpart on both sides — AccessKit has no direct
analogue for `Select`, `Copy`, `Paste`, `Cut`, or `Dismiss`, and the domain
has no analogue for AccessKit's scroll-direction or `Blur` actions — so both
methods return [`Option`] and yield [`None`] for an unmappable action rather
than inventing a wrong one.
