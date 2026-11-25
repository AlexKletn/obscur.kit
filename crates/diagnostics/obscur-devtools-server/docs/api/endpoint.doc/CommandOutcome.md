The result of successfully dispatching a [`CommandMessage`].

Every variant is a typed outcome — the endpoint never executes arbitrary
code; control and injection resolve to a plain acknowledgement (spec §7.7).
