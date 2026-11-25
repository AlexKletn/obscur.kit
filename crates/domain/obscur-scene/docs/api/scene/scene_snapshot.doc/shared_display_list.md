The display list as the shared handle the snapshot holds.

Identical in content to [`SceneSnapshot::display_list`]; this accessor exposes
the [`Arc`](std::sync::Arc) itself, for the callers that need the list's
*identity* — a backend keeping one frame's list alive next to the following
frame's, or a test proving that taking a snapshot copied nothing.
