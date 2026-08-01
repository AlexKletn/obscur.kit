A live handle to the surface a [`LayerApp`](super::LayerApp) is running.

# Why a handle and not `&mut` anything

The event loop owns the surface, and an application's command handlers run
*inside* it — so they cannot be given a reference to it without borrowing the
loop from itself. A handle sidesteps that entirely: a handler leaves a
request, and the loop picks it up at the top of the next turn. That also
means every change lands at a defined point in the frame rather than halfway
through one.

Clone it freely; every clone names the same surface.

# What it is for

Reconfiguring a live surface is how an application changes *how it is
presented* without restarting: a bar that grows, a panel that stops
reserving space and floats, a surface that moves from `Top` to `Overlay`.
None of layer-shell's placement properties need a new surface — band, anchor,
exclusive zone, margins and size are all live requests — so this is the whole
mechanism, and there is deliberately no "recreate" to reach for.
