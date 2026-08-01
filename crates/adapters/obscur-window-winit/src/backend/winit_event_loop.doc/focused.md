The window the compositor last told us holds keyboard focus, tracked from
the native `Focused` events. This is the only *observed* activation state
the backend has: winit grants focus asynchronously, so `activate` reads
this rather than assuming its request was honored (spec §6.3).
