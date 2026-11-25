The table of positional command registrations an application asked for, and the
nodes they are currently attached to (spec §2.6 links 1 and 3, §2.3 listeners).

# Why registrations are named, not passed as node ids

§2.6 routes a command *focused node → focus scope → ancestors → window →
application*, and [`UiRuntime`] implements all five — but the first four are
keyed by [`NodeId`], and a node id does not exist until a frame has built the
tree. An application configuring itself before `run` therefore has no id to
register with, which is exactly why only link 5 was reachable from `App`.

A [`SelectorTarget`] is the identifier that *does* exist beforehand: §7.5's
stable `test-id` / `automation-id`, written on the markup itself and resolved by
[`SelectorResolver`] against the live tree. So the application names the node,
this table resolves the name after every commit, and the registration lands on
whichever node currently carries it.

# Following the *name* across frames

A resolved binding is left alone while its node still answers to the name:
re-resolving per frame would cost a tree walk per registration, and
re-registering would be pointless for a handler (it replaces) and a leak for a
listener (they accumulate). It is released and resolved again when the node stops
answering — either it left the tree, or it is still there and no longer carries
the identifier, which is what happens when a reconcile patches an element in
place instead of replacing it.

That is the whole reason the check is [`answers_to`] and not merely
"is this id still live": the name is the contract, the node id is an
implementation detail of one frame's tree, and a handler must not keep answering
for an element that gave the name up. Confirming a *known* node still carries a
name is one node lookup; finding which node carries it is a tree walk, so this
pays the walk only when something actually changed. `NodeId`s are generational,
so a stale id can never alias a new node either.
