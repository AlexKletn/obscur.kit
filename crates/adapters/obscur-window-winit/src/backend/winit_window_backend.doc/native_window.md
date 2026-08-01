Returns a shared handle to the live winit window materialized for `id`,
or `None` if no OS window stands behind that id *yet* (spec §6.1).

# Why this exists, and why it is here rather than on the port

A host that renders needs the *platform* handle, not just an id: wgpu
builds a `Surface<'static>` from an `Arc<Window>`. Without some way to get
one, the port is unusable for exactly the hosts it exists to serve —
`create_window` hands back an id and the live window it names is
unreachable, which is why every windowed example calls
`event_loop.create_window` itself and this backend went a whole campaign
with no caller.

A `winit::window::Window` plainly cannot appear on [`WindowBackend`], a
domain trait that a WASM or headless backend also implements. But it would
**overclaim to say the domain has no vocabulary for a window handle at
all**: `raw-window-handle` 0.6 is platform-neutral, is already in this
tree (winit depends on it), covers Web as well as native, and is precisely
what wgpu consumes —
`SurfaceTarget::DisplayAndWindow(Box<dyn DisplayAndWindowHandle + 'window>)`
(`wgpu-29.0.3/src/api/surface.rs:280`). A port method handing back
`Box<dyn HasWindowHandle + HasDisplayHandle + 'static>` is a real
alternative, and it is the one that would let a host stay backend-agnostic
all the way to its swapchain.

It is **not taken here**, and the reason is a tradeoff rather than an
impossibility: it puts a platform-handle dependency into the pure
`obscur-windowing` domain crate, and it obliges every backend — including
the headless [`NullWindowBackend`](obscur_windowing::port::NullWindowBackend) —
to have an answer for a method most of them can only refuse. The escape
hatch is offered on this adapter instead: the one layer that is already
winit-specific, and which a host choosing this backend already names.
The cost of that choice is real and should be stated: a host's
swapchain setup is pinned to winit even though the rest of its
window handling is not.

# `None` is the deferred window, not an error

Creation is deferred (see the type-level docs): `create_window` mints the
id now and the [`WinitAppHandler`](super::WinitAppHandler) builds the OS
window on the next loop turn. Between those two moments the id is valid
and the window does not exist, so this answers `None` rather than block or
fabricate a handle. A host resolves it after the pump — the arrival of
[`WindowEvent::Created`] from `poll_window_events` is the signal that it
will now answer `Some`. An unknown or closed id also answers `None`.
