§6.1 Window model: "- create/destroy;" / "- owner/parent relations;"

Destroying an owner destroys the windows it owns; the child is absent from
the window list afterwards, with no dangling-handle panic.
