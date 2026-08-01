Exactly these contexts' inputs changed. Every other node's measure is served
from the retained [`MeasureCache`](crate::orchestration::MeasureCache) — see
[`LayoutCoordinator::layout_invalidated`](crate::orchestration::LayoutCoordinator::layout_invalidated)
for what that reuse rests on.
