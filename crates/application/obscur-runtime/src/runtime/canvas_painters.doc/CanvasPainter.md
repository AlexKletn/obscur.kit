What an application registers: a closure handed its own bounded region of the display list, and the properties of the node it is drawing.

The properties are the second argument because ONE painter serves EVERY node
that names it. Without them two `<canvas>` nodes sharing a painter would draw
the same picture with no way to differ — which is exactly what a built-in
control needs, since its widget registers one painter and every instance
parameterises it through its own props.

They cannot ride on the region: `CanvasRegion` lives in `obscur-scene`, which
sees geometry and nothing else. The painter type lives here, where both the
scene and the view model are in view.
