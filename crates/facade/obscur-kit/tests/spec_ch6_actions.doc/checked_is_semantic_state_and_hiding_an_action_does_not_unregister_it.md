§6.7: "- checked;" and "- visible;" in the Action model.

`checked` is semantic state, not a title decoration; `visible = false` hides
the action from rendered surfaces without unregistering it — the stable ID
still resolves and the handler is still invocable.
