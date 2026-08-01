A scroll of `delta_y` pixels at the pointer's position.

The position travels with the delta because scrolling is routed to the
container *under the pointer*, and a host that had to guess it — from the
hovered node's box, say — would send the notch to the wrong place the
moment the pointer sat over a child.
