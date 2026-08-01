A [`WindowBackend`] backed by winit 0.30 (spec §6.1-6.6, §2.7).

# Deferred, callback-driven creation

winit 0.30 is callback-driven: a real [`winit::window::Window`] can only be
built inside an [`ActiveEventLoop`](winit::event_loop::ActiveEventLoop)
callback, whereas this port is pull-based — `create_window` must return a
[`WindowId`] synchronously. The backend bridges the two by *deferring*: it
mints the id immediately and queues the request, and the
[`WinitAppHandler`] materializes the OS window on the next event-loop turn.
The port contract is honored (a valid id is returned); the window simply
appears one turn later.

# State split: owned records vs. shared winit state

The port's `window()` returns `&Window`, which cannot borrow through a mutex
guard. So the retained domain [`Window`] records — and the id counter — are
owned **directly** by the backend, giving `window()` a plain borrow. Only the
genuinely winit-side, handler-shared state (pending work queues, live winit
windows, the id maps, the event buffer) lives behind
`Arc<Mutex<BackendState>>`. Mutating operations update the owned record *and*
enqueue the corresponding winit work for the handler to perform.

# What this backend does *not* do

§6.2 requires a typed result naming *how* an operation was satisfied. The
gaps below are each reported at the call that hits it — but this list is not
exhaustive, and the residue is platform-conditional: winit documents
`set_outer_position` as unsupported on Wayland and Android, and
`set_decorations` as a no-op on iOS/Android/Web, while `set_position` and
`set_chrome_mode(SystemFrame)` still answer `native()` regardless of the
platform underneath. Detecting the compositor needs plumbing this backend
does not have, so on Wayland a `native()` from `update_window` can mean the
position was dropped. The gaps that *are* reported:

- **Owner relations and modality** have no winit primitive at all, so
  `create_window` refuses a request carrying either rather than returning an
  id for a window that would be top-level and modeless (see
  `unsatisfiable_relation`).
- **Transparency** is a creation-time attribute only; `update_window` reports
  a requested flip `conditional()` and leaves the record on the live value.
- **Exclusive / presentation fullscreen** are served by borderless
  (`conditional()`), and **kiosk** is refused (`Unsupported`) — see
  `presentation_outcome`, which also lists the §6.5 controls that stay
  unwired.
- **Custom chrome** only removes the server frame; the kit draws the rest, so
  client-side modes report `emulated()`.
- **Activation** is granted asynchronously by the compositor, which this
  pull-based port cannot wait on: `activate` reports `Activated` only for a
  window already observed focused, and `Denied` otherwise. A caller learns of
  a later grant from `WindowEvent::FocusGained`, never from `activate`'s own
  return value.

# Wiring a host into an event loop

```no_run
use obscur_window_winit::backend::WinitWindowBackend;
use obscur_window_winit::backend::UserEvent;
use obscur_window_winit::wake::EventLoopWaker;
use winit::event_loop::EventLoop;

let mut backend = WinitWindowBackend::new();
// The backend can be driven (create_window, update_window, ...) before or
// during the run; queued work is applied on each loop turn.

let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
// A cross-thread frame-scheduling waker, if desired:
let _waker = EventLoopWaker::new(event_loop.create_proxy());

// The handler shares the backend's state, so operations issued on `backend`
// are observed by the running loop.
let mut handler = backend.app_handler();
event_loop.run_app(&mut handler).unwrap();
```
