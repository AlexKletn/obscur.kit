One of the four box corners a `linear-gradient(to <corner>, …)` can point at.

Unlike a `to <side>` keyword (which folds to a fixed angle), a `to <corner>`
direction is **aspect-ratio-aware**: per [CSS Images §3.1] the gradient line is
angled so it points into the corner's quadrant and is *perpendicular to the
diagonal joining the two neighbouring corners* of the box. That angle depends
on the box's width/height, which is unknown at CSS-lowering time, so the corner
is carried symbolically here and resolved against the painted box by the scene
coordinator.

[CSS Images §3.1]: https://drafts.csswg.org/css-images-3/#linear-gradients
