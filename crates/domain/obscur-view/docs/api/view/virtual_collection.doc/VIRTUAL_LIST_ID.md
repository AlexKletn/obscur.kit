Reserved symbol for a virtual list's **name** — the handle that ties a marker
node in the view to the row builder the host registered (spec §4.9).

The same symbol is passed to
[`register_virtual_list`](../../../obscur_runtime/runtime/struct.UiRuntime.html#method.register_virtual_list)
and carried on the node, which is what lets one runtime drive several
independent lists: the marker says *which* list it is, and the runtime looks
up the builder, the realized window and the scroll offset under that name.
