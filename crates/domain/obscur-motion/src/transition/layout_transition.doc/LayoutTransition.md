A FLIP-style animation of an element's layout box (spec §4.17
"layout-affecting animations").

*FLIP* stands for **F**irst (the box *before* a layout change), **L**ast (the
box *after*), **I**nvert and **P**lay: given the before/after bounds, the
element is animated from the old box to the new one so a layout jump reads as
smooth movement. This type is the domain half — it holds the two bounds, an
easing curve and a play [`direction`](TransitionDirection), and interpolates
between them. Capturing the actual before/after geometry is the runtime's
job, performed in its pre/post-layout phases, which then feed those rects into
[`from_bounds`](Self::from_bounds) and [`to_bounds`](Self::to_bounds).
