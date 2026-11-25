Whether a listener called `prevent_default` during capture, target or bubble.

Reported out rather than only consumed here. §2.3's fourth stage is bigger than
the §2.6 routing chain this type performs: the runtime's own default actions —
the text edit, the activation, the focus move, the scroll — are what a `keydown`
listener usually means to refuse, and those are performed by the caller.
