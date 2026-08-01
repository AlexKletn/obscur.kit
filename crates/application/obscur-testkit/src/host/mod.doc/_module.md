Hosted code the runtime mounts and drives (spec §2.4): components a test
registers, and the plugin host facade.

Three of the four entities here are components, and they come as a set because
their point is the *relationship* between them. [`CounterComponent`] is the
healthy one, and its props type is deliberately shared by
[`GuardedComponent`] so that the same undecodable `step` fails both — which is
what lets a boundary test vary the policy while holding the failure fixed.
[`EnclosingComponent`] renders a failing guarded child, putting a second
boundary above the first.

[`PluginTestHost`] is still a placeholder: nothing in the suite drives plugin
activation through a harness yet, so the entity states the shape without
claiming the behaviour.
