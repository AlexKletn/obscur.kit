Recomputes the scroll-chain-adjusted view of the retained layout boxes that
[`layout_boxes`](Self::layout_boxes) exposes.

The internal [`boxes`](Self::boxes) stay in content space — paint/compose
([`SceneCoordinator::compose_with_images_scrolled`]) and hit-test (the
spatial index plus [`ScrollTargets::fold_point`])
read that field directly and apply scroll themselves. The projection is
[`ScrollOffsets`]'s to build; this resolves the containers it must be built
against from the live tree, styles and boxes, which that type deliberately
does not hold.

[`SceneCoordinator::compose_with_images_scrolled`]: crate::orchestration::SceneCoordinator::compose_with_images_scrolled
[`ScrollTargets::fold_point`]: crate::runtime::ScrollTargets::fold_point
[`ScrollOffsets`]: crate::runtime::ScrollOffsets
