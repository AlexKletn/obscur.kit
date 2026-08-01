Builds a [`WinitAppHandler`] sharing this backend's winit-side state.

Hand the returned handler to
[`run_app`](winit::event_loop::EventLoop::run_app); it observes and
services every operation issued on this backend. See the type-level docs
for a full wiring example.
