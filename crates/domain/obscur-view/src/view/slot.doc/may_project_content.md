Whether a node carrying `component` could ever project authored content —
the reconciler's second gate for retaining a host's children as §2.12 slot
content.

The first gate is [`assigns_slots`]: children that name a slot are kept
because they named one. That is not enough for the **default** slot, which is
the one content reaches by naming nothing: `<button>Save</button>` assigns no
slot and its only child is a text run, which can carry no assignment at all,
so a gate that reads the children alone would drop exactly the content the
default slot exists to project.

The gate that is left is the host's own id, and it is a good one because it is
also the honest question: content can only be projected by a component, a
component is only ever mounted under an id in the widget range
(`0x00D0_xxxx`) or the top-bit range a user component hashes into, and every
`div`, `span` and text run in a document is a primitive below both. So the
overwhelming majority of a tree pays nothing, and what pays is one clone of a
child list per widget or user-component node — bounded by that control's own
content, which is small for every shipped control.

It answers about the *id*, not about the registry, for the reason the
reconciler's slot-content pass gives: whether a node actually hosts a
component is known only after the plan that pass produces has been synced
against a registry the reconciler cannot see, and waiting for it would drop
the authored content of every component's opening frame. An id that mounts
nothing therefore pays the clone and nothing reads it — the price of keeping
the decision on data the reconciler has in front of it.
