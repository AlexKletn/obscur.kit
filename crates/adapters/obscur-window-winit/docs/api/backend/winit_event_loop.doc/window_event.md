Routes a native window event to its domain window, converting it.

Only lifecycle/geometry/focus/scale/occlusion events convert to a domain
[`WindowEvent`]; keyboard and pointer input are the interaction crate's
concern and out of this port's `poll_window_events`, so they are not
buffered here (the [`KeyboardConverter`](crate::conversion::KeyboardConverter)
/ [`PointerConverter`](crate::conversion::PointerConverter) feed a separate
input path in a later sub-step). An event for an unknown winit window is
dropped.
