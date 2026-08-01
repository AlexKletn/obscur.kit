Handles a cross-thread wake-up by requesting a redraw of every live
window (spec §3.5).

The frame scheduler sends [`UserEvent::WakeUp`] purely to break the loop
out of its wait; translating it into redraw requests is the honest way to
turn "a frame is due" into winit's redraw-driven rendering model.
