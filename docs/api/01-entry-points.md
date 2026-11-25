# 01 — Entry points

Three ways in, in increasing order of how much you own yourself.

| | You supply | You own | Feature |
|---|---|---|---|
| [`App`](#app) | a view builder and handlers | nothing | `render-vello` + `window-winit` and/or `shell-layer` |
| [`UiRuntime`](#embedding-a-uiruntime) | the surface and the loop | everything around a frame | any |
| [`HeadlessHost`](#headless) | nothing | a deterministic driver | any |

`App` covers both an ordinary window and a **desktop surface** — a panel, dock,
bar or overlay. One call chooses:
[`App::layer(settings)`](#opening-a-desktop-surface-instead-of-a-window).

---

## `App`

`obscur_kit::App` — the one-call entry (`src/app/app.rs`). It owns the surface,
the GPU swapchain, the event loop, the text stack and the frame pipeline. It
needs `render-vello` plus at least one surface backend — `window-winit`,
`shell-layer`, or both — and is absent from the headless default build.

```rust
use obscur_kit::App;

fn main() -> Result<(), obscur_kit::AppError> {
    App::new("Hello")
        .with_size(480.0, 240.0)
        .stylesheet(".root { background-color: #202430; color: #eef }")
        .view(|| build_view())
        .run()
}
```

### The whole surface

Every method is a consuming builder returning `Self`, except `run`.

| Method | |
|---|---|
| `App::new(title: impl Into<String>) -> Self` | starts at **800×600** logical, app id `"obscur-kit"`, no styling, no root |
| `app_id(impl Into<String>) -> Self` | the Wayland `app_id` / X11 `WM_CLASS`. This — not the title — is what a compositor rule, a taskbar entry and a `.desktop` file match on. Give every app a distinct one |
| `with_background_blur(bool) -> Self` | requests compositor blur behind the entire surface. The default is off. See [08 — Rendering](08-rendering.md#wayland-background-blur) |
| `with_renderer(RenderBackend) -> Self` | selects the GPU renderer for a window or layer surface. See [08 — Rendering](08-rendering.md#lyon-renderer) |
| `with_size(f32, f32) -> Self` | initial size in **logical** pixels |
| `with_min_size(f32, f32) -> Self` | optional minimum content size in logical pixels |
| `with_max_size(f32, f32) -> Self` | optional maximum content size in logical pixels |
| `with_resize(bool) -> Self` | enables user resizing (the default), including automatic grips along rounded edges of undecorated windows where the backend supports them |
| `prefer_opengl(bool) -> Self` | tries a hardware OpenGL adapter before the normal desktop adapter selection |
| `stylesheet(&str) -> Self` | appends a sheet. Callable repeatedly; later sheets cascade over earlier ones. This is the *initial* set — to replace it while running, see `control()` |
| `view(impl Fn() -> View + 'static) -> Self` | the root builder, re-invoked per frame |
| `view_with(impl FnMut(&FrameInfo) -> View + 'static) -> Self` | the root builder, given the live viewport and scale |
| `screen(&str) -> Self` | the root as `.okui` markup: compiled to UI IR at `run`, then re-lowered every frame against `screen_data()` |
| `screen_data() -> ScreenData` | the table a `screen`'s `{bindings}` read. Plain application state — write it before `run` and from a handler |
| `control() -> AppControl` | the handle onto this app's running runtime: positional handlers, listeners, `set_stylesheets`, a queued `with_runtime` escape hatch |
| `on_command(InternedSymbol, handler) -> Self` | a handler for a command name bound in the view or the sheet, at the **window** level (§2.6 link 4) |
| `on_command_at(SelectorTarget, InternedSymbol, handler) -> Self` | the same, **at a node** named by `test-id` (§2.6 links 1 and 3). Claims the command and stops the chain |
| `add_event_listener(SelectorTarget, InternedSymbol, ListenerPhase, listener) -> Self` | a §2.3 listener that observes and steers a command in `Capture` or `Bubble` |
| `on_scroll(impl FnMut(f32) + 'static) -> Self` | wheel deltas **no `overflow: scroll` container consumed** |
| `painter(impl Into<String>, painter) -> Self` | a canvas painter under the CSS-visible *name*, which is what `-obscur-painter: <ident>` reaches. See [04 — Styling](04-styling.md#vendor-properties) |
| `painter_id(InternedSymbol, painter) -> Self` | the same, under the *symbol* a node names in its own property — including a shipped widget's (`Slider::PAINTER`, `Progress::PAINTER`), which this replaces for the whole application |
| `virtual_list(InternedSymbol, impl FnMut(usize) -> ViewNode + 'static) -> Self` | the row builder for a list the runtime windows |
| `on_row_click(InternedSymbol, impl FnMut(usize, u32) + 'static) -> Self` | `(model index, click count)` for a row of that list |
| `on_row_long_press(InternedSymbol, impl FnMut(usize) + 'static) -> Self` | a held row of that list |
| `run(self) -> Result<(), AppError>` | opens the window and runs to completion |

`app.control().window_state()` returns the desktop window's last reported
`WindowState`. Read it inside a view builder to update window chrome when
the window is maximized, restored, fullscreen, or focused. The snapshot also
carries the logical size, scale factor, resizability and minimized state.
Wayland leaves `minimized` unset. Use `FrameInfo::viewport` to lay out the
frame currently being built during an interactive resize.

`view`, `view_with` and `screen` set the same slot: **the last one called wins.**

### A handler takes nothing, or takes the event

`on_command`, `on_command_at` and `add_event_listener` accept either shape
(`IntoCommandHandler`):

```rust
.on_command(SAVE, || save())                              // takes nothing
.on_command(PICK, |e: &mut UiEvent| pick(e.target()))     // takes the event
```

The event form is how §2.3's model reaches an application: `target()`,
`current_target()`, `pointer()` / `key()`, `trust()`, and
`stop_propagation()` / `prevent_default()`. A one-argument closure needs its
parameter annotated — inference cannot pick the marker and the signature at once.
Until this widened, the handler was an `FnMut()` given nothing, so a handler had
to *capture* everything it might need.

`on_row_click`, `on_row_long_press` and `on_scroll` stay outside that model: the
runtime calls them straight from its pointer path rather than dispatching a
command, so no event is delivered and no listener sees them.

### Reaching the running runtime

The loop owns the runtime and your handlers run inside it, so nothing hands out a
`&mut UiRuntime`. `App::control()` is the way in — a cloneable handle a handler can
capture, whose requests the runner applies at the next frame boundary:

```rust
let control = app.control();
control.set_stylesheets(vec![theme_css()]);   // §4.12 explicit invalidation
// `with_runtime` takes `impl FnOnce(&mut UiRuntime) + 'static`, so the closure
// must return `()` — brace-and-semicolon anything that answers, as this does.
control.with_runtime(move |rt| { rt.scroll_into_view(node); });
control.bound_node(&SelectorTarget::test_id("save")); // did that name resolve?
```

`set_stylesheets` is the reason a theme switch no longer costs a rebuild: `App`
used to collect the authored sheets once and never re-supply them, and never yield
the runtime, so §4.12's "invalidated **explicitly**" was unreachable from a
standalone application.

### `FrameInfo`

```rust
pub struct FrameInfo {
    pub viewport: Size,  // logical (CSS) pixels
    pub scale: f32,      // device-pixel ratio
}
```

Handed to a `view_with` builder each frame. This is what lets an application
window its own list to what is visible instead of building a node per item — the
alternative to `virtual_list`, for when the app wants to own the arithmetic.

### Application components

Call `App::component(registration)` for each custom component. Both window
and layer runners install these registrations before building the first
view. A duplicate identifier, including a built-in identifier, returns
`AppError::ComponentRegistration` during startup. For `App::screen`, provide
the matching source contract through `App::screen_contract`, typically using
the component's derived `contract_surface()`.

### System theme

`App::system_theme(provider)` opts into a startup snapshot from any
`AppearanceProvider`. The window and layer runners use the same path.
Author CSS can read the reported values through `--obscur-sys-*` variables
and override them through the normal cascade. A host can send a later
snapshot through `AppControl::set_system_appearance`. If the provider
supports a watch, its notifications wake the event loop and apply the latest
snapshot. The subscription ends when the application exits.

### `AppError`

`ScreenParse(Vec<String>)` · `ScreenCompile(Vec<String>)` · `EventLoop(String)` ·
`Window(String)` · `Gpu(String)`.

A GPU or window failure exits the loop cleanly and is reported here rather than
leaving a blank window: `run` returns the reason.

### What `App` wires for you

On `resumed`, in order: the UA stylesheet, the author sheets through the CSS
pipeline, image and vector resources, the blocking executor, the real text stack
over the host's fonts, the Vello renderer and surface, and the winit event
converters. Then per event: pointer/touch conversion, wheel routing, keyboard and
IME, motion advance, virtual-list windowing, and text entry offered per focus.

Two environment variables it reads:

- `OBSCUR_FRAME_STATS=1` — per-phase frame timings every 60 presented frames
- `OBSCUR_NO_VSYNC=1` — see [08 — Rendering](08-rendering.md)

---

## Opening a desktop surface instead of a window

One call. `App::layer(settings)` puts the same application on the **desktop**
rather than in the window stack: a panel, a dock, a bar, a wallpaper, a
notification, an on-screen keyboard. No title, no decorations, no place among
the windows — anchored to screen edges, in one of four depth bands, optionally
reserving space that maximised windows then keep clear of.

```rust
use obscur_kit::App;
use obscur_kit::windowing::layer::LayerSettings;

App::new("status bar")
    .app_id("com.example.bar")
    .layer(LayerSettings::top_bar(36))
    .stylesheet(CSS)
    .view(|| bar(&state.borrow()))
    .run()?
```

**That method is the whole of the difference.** Everything else is the `App` you
already know — the same `View`, the same CSS, the same commands, virtual lists,
row clicks, touch and gesture handling, the same runtime underneath. The
application's `app_id` becomes the surface's identity to the desktop; there is
no second name to set.

One method does not carry over: `view_with` is a windowed root, and using it
with `layer` returns `AppError::FrameAwareLayerRoot`. A desktop surface is sized
by the platform *before* its first frame, so there is nothing a frame-aware
builder would learn that `view` does not already have.

### The settings

`obscur_kit::windowing::layer::LayerSettings` — platform-neutral by
construction. No protocol vocabulary, no wire encoding, no sentinel values.

| | |
|---|---|
| `top_bar(h)` / `bottom_bar(h)` / `left_bar(w)` / `right_bar(w)` | a bar reserving exactly its own thickness |
| `overlay()` | whole-screen, above everything, reserving nothing and ignoring what others reserved |
| `new()` | a floating surface in the panel band, anchored to nothing |
| `with_band(LayerBand)` | `Background` · `Bottom` · `Top` · `Overlay` |
| `with_anchor(LayerAnchor)` | anchoring both edges of an axis **stretches**, and overrides the size on it |
| `with_reservation(LayerReservation)` | `None` · `Reserve(px)` · `Ignore` |
| `with_margin(LayerMargins)` · `with_focus(LayerFocusPolicy)` · `with_display(DisplayId)` | |
| `with_pointer(LayerPointerPolicy)` | `Interactive` (the default) or `Transparent` — a surface that only *shows* something lets clicks through to whatever is behind it |
| `with_size(w, h)` | an axis you leave unset is the platform's to choose — `None`, never a zero |

A display is chosen by `DisplayId` — the same id
[`WindowBackend::displays`](#embedding-a-uiruntime) hands out — rather than by a
connector name, because a name that identifies a monitor on one platform
identifies nothing on the next.

### What the platform can actually do

Desktop platforms diverge here more than they do over windows, so the port says
so rather than accepting a setting and dropping it.
`obscur_windowing::port::LayerBackend::supports` answers a `SupportLevel` for
each `LayerFeature`: `Bands`, `Reservation`, `FreeAnchor`, `ExclusiveKeyboard`, `OnDemandKeyboard`,
`DisplaySelection`, `NegativeMargin`, `FractionalScale`, `InputTransparency`, `Popup`, `TextInput`.

The one that most often comes back short is `Reservation` — taking space from
the desktop is an ordinary request on some platforms and has no public API at
all on others.

### Reconfiguring a running surface

The loop owns the surface and your handlers run inside it, so they cannot hold a
reference to it. `App::layer_control()` is how they reach it: a handler leaves a
request, the loop applies it at a defined point in the frame.

```rust
let app = App::new("bar").app_id("com.example.bar").layer(LayerSettings::top_bar(36));
let control = app.layer_control();
app.on_command(FLOAT, move || control.reconfigure(floating_settings()))
    .run()?
```

It also reports the surface's real `size()`, `scale()` and `state()`
(`Configuring` → `Ready` → `Closed`), and `close()` takes it down.

**No placement change needs a new surface.** Band, anchor, reservation, margin
and size are all live requests, which is why there is no "recreate". The one
exception is the display: an output is chosen when the surface is created, so
moving it comes back `conditional()` rather than `native()`.

### Waking the loop from outside

The loop sleeps until the platform has something to say. Nothing else makes it
turn — not a worker thread, not a timer. `App::waker(…)` takes the handle that
does, and it is `Send`:

```rust
let waker = LayerWaker::new()?;
let ticker = waker.clone();
std::thread::spawn(move || loop {
    std::thread::sleep(Duration::from_secs(1));
    update_shared_state();
    ticker.wake();          // the loop turns and rebuilds the view
});

App::new("bar").layer(LayerSettings::top_bar(36)).waker(waker).view(bar).run()?
```

A wake means "reconsider the interface" and nothing finer — the waker does not
know why it was rung. A worker that needs to say *what* changed puts that in the
state the view reads, where the view will actually see it.

### Where it runs

Today one backend implements the port: `wlr-layer-shell`, which wlroots
compositors (Sway, Hyprland, river, Wayfire) and KDE implement. **GNOME has
declined the protocol**, and there `run` returns `AppError::LayerUnsupported` —
a capability answer to branch on, not a crash. The fallback is an ordinary
window, reached by not calling `layer`.

The settings and the port are written to outlive that backend: nothing in
either names Wayland, and an appbar or an `NSWindow`-level backend would
implement the same `LayerBackend` and answer the same `LayerFeature` roster.

### Scale

Fractional scaling is used where the compositor offers `wp_fractional_scale_v1`
and `wp_viewporter`: the buffer is rendered at the true scale (1.6, not 2) and a
viewport maps it onto the logical size. Without both, the integer path applies
and a fractional output rounds up — which is what
`LayerFeature::FractionalScale` answers `Emulated` for.

---

## Embedding a `UiRuntime`

`UiRuntime` is the whole toolkit minus the window. It has no display, no event
loop and no clock — it takes time and input as data. A host builds one, installs
the ports it wants, and calls a frame method.

```rust
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::runtime::UiRuntime;

let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(800.0, 600.0));
let result = rt.render_now(&view, &sheets);
let snapshot = result.snapshot();      // hand to a renderer
```

The minimum useful host is four calls: `new`, install a text stack (otherwise
text measures but paints no glyphs), `render_now`, and hand the snapshot to a
renderer. [`KitBootstrap`](10-hosting.md) does the installing.

Full method list: [03 — The runtime](03-runtime.md).

## Plugin integration points

Enable `plugins` to use `obscur_kit::plugins`, including the host and package
loader. Enable `plugin-process` for the process executor and framed transport.
The `full` feature includes `plugin-process`.

The host application defines its own identifiers with `ExtensionPoint::new`
and publishes them on a `PluginManager`. A plugin uses
`PluginRegistration::contribute_view` to stage a contribution for one of
those points. Each contribution carries presentation metadata and a
`View` factory. The application selects the views for a point with
`contributions().views_at(&point)` and inserts their rendered views where
they belong in its UI.

For routed contributions, use `publish_with` and return a static `Route`
from the application policy. That policy can reject submissions or return
no route for a view that the application will place directly. Use
`navigate` to resolve routes and `current_view` to build a routed plugin
view. Activation publishes all contributions together after checking
points, duplicate IDs, application policy, and route conflicts.

Deactivation withdraws plugin views and routes before teardown. The router
retains history, so the host can supply its own fallback when
`current_view` has no matching plugin.

### Validating plugin packages

`PackageValidator::load` accepts a `PackageReader` and returns a verified
package in memory. `TarPackageReader` handles TAR, TAR.GZ, and TAR.ZST.
Use `ZipPackageReader` for single-volume ZIP with UTF-8 paths and stored or
Deflate entries. The ZIP reader rejects encrypted entries, ZIP64 archive
directories, and earlier footer signatures. The TAR.ZST decoder caps its
window at the expanded size budget rounded up to a power of two, within
1 KiB to 128 MiB.
A package carries `manifest.json` at its root. Schema version 1 identifies
the plugin, its execution kind, the supported Plugin API and UI IR versions,
its entrypoint, and the SHA-256 digest of every payload file.

The application sets the API versions, capability allowlist, dependency
inventory, and byte budgets on the validator. Capabilities are denied by
default. Required and service-only dependencies must appear in the host
inventory at a compatible version. Optional dependencies may be absent.
The reader rejects path traversal, links, unsupported archive records,
duplicate paths, and packages over the configured limits. Validation
requires a hash for the entrypoint and every other payload file.

A validated package exposes its manifest and verified entrypoint bytes.

### Extracting a verified plugin payload

Call `ExtractionPlan::new(&package).extract_in(parent)` with an existing
host-managed parent directory. Keep the parent and its ancestors protected
from replacement. The new directory contains the verified payload files
and exposes the manifest-selected entrypoint and other files by package path.
Extraction rejects destination names that are unsafe on Windows as well as
filesystem conflicts. Unix permissions are limited to owner access, with
regular files created without execute permission. On systems using inherited
permissions, restrict the parent directory to the intended users.

Keep the returned `IsolatedDirectory` alive while using its paths. On failure
or drop, the kit attempts to remove the extracted directory. Call `close`
to receive cleanup errors explicitly.

### Publishing an installed package

On Unix and Windows, open an existing dedicated host-owned directory with
`InstallationStore::open`. Use `stage(&package)` to write a validated package
beside the current version. Keep the store and its contents protected from
untrusted writes. Open the staged entrypoint through `candidate()` for host
checks and trial activation.

Call `commit()` after the trial succeeds. Commit atomically replaces the
current-generation marker and fails if another transaction published the
same plugin after staging. Dropping a staged transaction removes its candidate.
`current(id)` reopens the published generation after a restart. Committed
generations stay on disk so consumers holding old paths can finish using them.

This transaction publishes the package on disk. The host coordinates
dependency checks, state migration, and UI handover. Atomic publication
covers concurrent access. Recovery after power loss and collection of
retired generations are separate work.

### Resolving plugin dependencies

Pass the host's selected manifests to `DependencyResolver::resolve` before
activation. The graph exposes the provider-first `activation_order` and the
resolved dependencies of each plugin. Required and service-only providers must
be present. Missing optional providers contribute no edge. A present provider
must satisfy the requested range, including for optional dependencies.

The resolver accepts one version per plugin ID and reports duplicate entries,
missing providers, version conflicts, and cycles. Use
`DependencyGraph::with_update` to test a proposed provider version against
all consumers before replacing it. These calls return a plan for the
application to apply.

### Persisting plugin state

`StateStore` is the host port for versioned, opaque state snapshots.
On Unix and Windows, `FilesystemStateStore` uses a dedicated host-managed
directory and a `StorageQuota` per snapshot. `compare_exchange` publishes
the candidate atomically when the stored snapshot still matches the one
the host loaded. A conflict returns false. Pass no expected snapshot to
create state only if the namespace is empty.

`StorageNamespace::new` addresses the installation's global state.
Select `StateScope::Instance` with a stable host-assigned instance ID or
`StateScope::Cache` with `with_scope`. `CacheStore::clear_cache` clears only
the cache of the given installation. Global and instance snapshots remain
available. Secret values belong in the secure storage service.

The filesystem adapter synchronizes the file before replacing it. It relies
on filesystem locking and atomic replacement and does not guarantee that
a rename survives power loss.

### Authorizing plugin service calls

The host uses `CapabilityIssuer` to issue opaque handles after policy approval.
Each handle binds a plugin principal to one service version, permitted
operations, and an optional deadline. The host calls `authorize` with the
authenticated caller and the actual operation before dispatch.

Only trusted host code delegates a handle. A derived handle can select a
different principal, but its operations and lifetime must fit within its
source grant. Call `CapabilityRevoker::revoke_principal` on unload to revoke
the principal's grants and their descendants. Handles from a different issuer
are rejected. Host integration supplies policy decisions and sandbox transport.

### Dispatching typed plugin services

Use `ServiceRegistry<Request, Response>` for an in-process Rust service
contract. Register each provider with its service version, supported
operations, and async handler. Issue a `ServiceHandle` after checking the
consumer's dependencies and policy. Dispatch authenticates the caller and
operation before passing a typed request to the provider. An unavailable
registration yields `Unsupported`. A Rust panic that unwinds while creating
or polling the handler future is reported as `ServiceDispatchError::Panicked`.
The JSON bridge returns `Failed` and releases the call slot. Process aborts
remain outside this boundary.

Obtain `registration_revoker(service, version)` to withdraw one registration
while calls are pending. Its callers receive `Unsupported` and new grants
are blocked. After dispatches finish, call `unregister_provider` on unload.
Old handles cannot dispatch to a replacement, even if it registers the same
service ID and version.
Delegation and revocation use the same capability issuer.

Transport adapters can use `JsonServiceBridge` with serializable request and
response types. The adapter supplies the already resolved handle, authenticated
caller, operation, and JSON payload. The bridge authorizes before decoding,
enforces host-defined request and response byte budgets, and returns stable
error codes without forwarding provider diagnostics. Replies that exceed
the budget may follow a completed provider operation.

For a transport connection, create a `ServiceSession` with the identity
authenticated by the adapter and a host-defined handle limit. The host
issues numeric handles through the session after checking policy.
`JsonServiceBridge::dispatch_session` uses the session's own grants and
principal. Unknown and foreign handles return `Denied`. Release individual
handles or drop the session on disconnect to release all its grants.
Keep a `ServiceSessionRevoker` from `session.revoker()` when host policy needs
to revoke the session while dispatch is pending. Revocation wakes pending
handlers, returns `Denied`, and blocks new grants and calls. It is separate
from closing the underlying streams and cannot preempt synchronous code.
Use `session.handle_revoker(id)` to cancel future and pending calls for one
handle while other handles remain usable. Call `release(id)` to reclaim
that handle's slot in the session table.
The adapter authenticates the connection. Hosts may use `with_call_limit`
with a `ServiceCallLimit` to cap attempts in a rolling window. Each decoded
call counts before handle lookup, including calls with invalid handles.
When the budget is depleted, dispatch returns the existing `Denied` status.
`with_in_flight_limit` also caps concurrent calls within one session.
Excess calls receive `Denied` without queueing. Completion, failure, and
cancellation return their slot to the session.

Use `ServiceFrameCodec` for sequential request/response streams. Every frame
starts with a four-byte big-endian body length, checked against the adapter's
budget before the body is read. Version 1 calls carry a one-byte version,
eight-byte big-endian handle, two-byte big-endian UTF-8 operation length,
the operation, and the JSON payload. Replies carry the version and a one-byte
status, followed by JSON. Status 0 carries a response and status 1 carries
a stable error code. The bridge dispatches decoded calls with
`dispatch_frame`.

`FramedTransport` attaches an authenticated session to paired streams, such
as the stdout and stdin of a host-launched plugin. Run `serve_next` on a
transport worker because stream reads and writes block. It serves one call
and flushes its reply. EOF, framing or I/O errors, and cancellation during
dispatch drop both streams and the session. With `serve_next_until`, the host
supplies a timer future after the frame read. Expiration drops a pending
handler, releases its slot, and returns `Failed` while leaving the stream
available. The timer cannot preempt synchronous code inside a handler poll
or undo earlier effects. It does not bound the blocking frame read.

On Unix and Windows, `ProcessExecutor::spawn` runs a host-approved
`std::process::Command` with piped stdin and stdout. Pass the initial session
handle IDs to the child through the launch configuration. Reserve stdout
for frames and configure stderr for diagnostics. Use `try_wait` to poll,
`wait` to close the streams and wait for normal exit, or `stop` to close,
terminate, and reap the child. Dropping the executor attempts the same cleanup.
A host watchdog can call `stop_handle().stop()` from a separate thread while
the transport is blocked. The handle terminates and reaps the direct child.
Use `ProcessWatchdog::arm` for a one-shot deadline. Keep the watchdog alive
during the monitored operation, then call `disarm` to cancel the deadline
and learn whether it already expired.

Use `shutdown_until` on the process worker to close the transport, signal
end of input to the child, and allow it to exit before the host deadline.
The watchdog terminates and reaps the child on expiration. The method
reports that path as a timed-out I/O error.

For a cooperative shutdown concern, pass the process's authenticated control
connection to `ProcessShutdownChannel` and attach it to the executor with
`with_shutdown_channel`. The executor or supervisor can deliver
`shutdown_requested` over this separate TCP stream with one deadline for
the exchange. The application can use the returned concern when choosing
the final deadline for `shutdown_until`.

On Linux, use `spawn_in_process_group` to stop both the child and descendants
that stay in its group. The executor stops any remaining group members
before reaping a normally exited leader. The executor must be the sole
reaper for that child. Other platforms and ordinary `spawn` use direct
child control, so inherited pipe copies may remain open. The host supplies
sandboxing.

Enable `plugin-native` to use `NativePlugin::load` with a validated native
package and a private extraction parent. The application explicitly trusts
the binary and its native dependencies before opening it. Use the verified
ID and version with `PluginManager::activate_with_version`. Deactivation
runs the lifecycle hook while the library remains mapped until process exit.
Obtain a non-owning handle with `NativePlugin::handle` before activation.
The handle can exchange messages while the plugin is active if the guest
exports its dispatch hook using `export_native_dispatch!`.

For a native guest that exports `obscur_plugin_bind_host_v1`, use
`NativePlugin::load_with_services` with the installation's service session.
The guest uses the same framed service packets as the other executors.
Allocate `HostFunctionTable::SERVICE_PACKET_BYTES` for the reply. Calls
are scoped to binding and initialization on the host thread. The session
is revoked before the deactivation hook.

### Running a verified WASM module

Enable `plugin-wasm` to add `WasmExecutor`; it also enables `plugins`. It loads
the verified entrypoint from a WASM `PluginPackage` and instantiates at most
one memory and one table. Modules are binary and may not declare a start
function. Host imports are disabled by default. `WasmLimits` sets the module size, memory and table limits, and
fuel for each invocation. The engine remains internal to the adapter.

`call_i32` invokes a host-selected export with signature `(i32) -> i32`.
For message-based calls, `exchange` copies bytes through the version-1 guest
buffer. Export `memory` and two nonnegative i32 globals named
`obscur_plugin_buffer_ptr` and `obscur_plugin_buffer_capacity`. The host copies
the request there, calls `obscur_plugin_dispatch_v1(i32) -> i32` with the byte
length, then copies the indicated reply from the same buffer. The host
validates the buffer range and both message budgets before copying data.
The ABI carries opaque bytes, so the host and guest choose the message schema.

A trap or invalid reply length faults the instance. Load a fresh instance
from the verified package to restart.

Use `WasmPluginInstance::activate` to run the guest lifecycle. In addition to
dispatch, the guest exports `obscur_plugin_initialize_v1` and
`obscur_plugin_deactivate_v1`, both with signature `(i32) -> i32` and the same
buffer contract. Activation checks the exports before running guest code
and returns the initialization reply for the host to validate before
publishing contributions. `deactivate` consumes the instance and returns the
shutdown reply. Dropping an instance frees guest memory without invoking a
guest hook.

WASM guests may export `obscur_plugin_shutdown_requested_v1` with the same
signature and buffer contract. The request contains eight little-endian bytes
for the remaining time in milliseconds. A zero-length reply means no concern.
Otherwise the reply starts with eight little-endian bytes for the requested
time in milliseconds, followed by a UTF-8 message. The host caps the requested
time to its remaining budget. The guest call uses its normal total fuel
limit and checks the deadline between execution slices. An individual
guest operation may run until the next check. A timeout faults the instance.

For manager integration, create a `ManagedWasmPlugin` with the validated
package, initial bytes, and a trusted host decoder for the initialization
reply. The decoder stages contributions through `PluginRegistration` after
validating the payload. `activate_wasm` and `stage_wasm_handover` use the
verified manifest ID and the manager's normal registration and route
checks. Deactivation and successful handover run the appropriate guest
shutdown hook. The host can use the shared `PluginUi` decoder or its own
application-specific format.

Call `ManagedWasmPlugin::handle()` before handing the plugin to the manager
to retain `exchange` access on the manager thread. Each handle follows its
own instance through trial activation and teardown. It becomes inactive if
that instance is rejected, rolled back, replaced, or deactivated.

### Exchanging UI and events with a sandboxed plugin

`PluginUi::decode` reads a version-1 initialization reply and stages it with
`PluginUi::stage` through the normal plugin registration transaction. The
JSON envelope has `version: 1` and a `views` array. Each item specifies an
application-defined `point`, local `id`, `title`, optional `order`, and
`ir` as a JSON byte array containing an encoded OKIR document. The host
sets the byte and total node budgets. The decoder also bounds recursion.
An optional `contract` string specifies the required SemVer range for the
point. The application publishes the contract version with
`PluginManager::publish_versioned`. Activation skips views whose declared
range does not match and mounts the compatible views.
WASM package manifests can also declare `compatibility.extension_contracts`,
a map from point IDs to SemVer ranges. Both the package range and the view
range must match when both are present.

The first profile supports fragments, text, `div`, `span`, and `button`,
with literal classes and button click bindings. The host supplies fresh
command symbols reserved for the guest. Bind `PluginUi::commands()` in the
runtime and call `event_message(command)` to encode a version-1
`ui_event` identifying the point, view, node, event, and guest action.
Pass those bytes to the managed guest with `exchange`. The application
owns command binding and when to pump service requests. The kit supplies
the common decoder and event format for all compliant plugins.

A guest can return a full UI snapshot after an event or a service reply.
Use the same envelope with a strictly newer `revision` and call
`PluginUi::apply_update` on the handle that staged the views. Initialization
defaults to revision zero. Updates keep the same contribution IDs, points,
titles and order. The decoder validates the entire snapshot before swapping
the rendered views and command table. Unchanged events retain their host
command symbols. After a successful update, the host can compare
`commands()` before and after, unregister removed callbacks with
`UiRuntime::remove_command_handler`, bind new commands, and request a render. Older and duplicate snapshots are ignored.

### Calling host services from WASM

Enable the guest import with `ManagedWasmPlugin::with_host_calls(max_bytes)`
or the explicit executor/instance constructors. The only supported import
is `obscur_host.submit_v1(i32 ptr, i32 len) -> i32`. It copies one complete
`ServiceFrameCodec` packet from exported guest memory and returns 0 on
acceptance, 1 for invalid memory or size, and 2 if a request is queued.
A trap or invalid guest reply discards the queued request.

After a successful guest invocation, the host takes the packet with
`take_host_call()` and awaits `JsonServiceBridge::dispatch_packet` using the
`ServiceSession` that the host created for this installation. The guest
provides a handle and operation, while the session supplies caller identity
and permissions. Send the encoded response back in a subsequent
`exchange`. The guest protocol distinguishes that reply from other messages.
Dispatch initialization requests only after host validation and publication.
Pending requests are discarded on teardown.

For managed plugins, `WasmServiceDriver` binds a guest handle to one host
session and checks that the session's plugin ID matches the manifest.
Use `exchange` to enter the guest and `pump_next` to dispatch and deliver
one pending request. A failed or cancelled delivery revokes the session,
because a service may already have performed side effects. The driver
leaves scheduling of subsequent requests to the host. Deactivation or
rollback wakes a pending driver, cancels its service future, and revokes
its session without delivering a reply to the withdrawn instance.

Pass a `JsonServiceRouter` to `pump_next` when a session has grants for
several service contracts. Mount each service ID and version with its own
typed `JsonServiceBridge` and byte budgets. The router chooses from the
host-issued handle, and applies the same session limits and authorization
as a single bridge.

### Providing plugin UI inside an application

The host application chooses which extension points to publish. A plugin
provides its own UI contribution and application logic. For example, one
application can define a point for preferences pages and another can
define a point for project views. Both use the same registration API.

A plugin handles its own actions. When sandboxed UI logic needs access to
a system API, the application can publish a domain-specific service and
grant the plugin the operations it needs through the generic service
registry. The application owns the service contract and implementation.

### Using credentials from secure storage

Implement `SecureStorageBackend` with a protected store, such as an OS
keyring, keyed by `PluginPrincipal` and credential key. Keep the resulting
`SecureStorageService` in trusted host code. Host policy decides whether to
issue an opaque lease and supplies the canonical service, operation, origin,
and expiration. Only the lease is passed to the plugin.

A trusted host service calls `with_secret` with the authenticated caller and
the destination of the operation. An expired, revoked, or mismatched lease
is denied. Revoking a lease invalidates all its copies for future uses.
The backend and trusted operation handle secret bytes and must redact them
from diagnostics. The kit supplies the port and authorization checks. The
host supplies secret entry UI and the concrete keyring backend.

### Migrating plugin state before activation

Load a snapshot from host storage into `PluginStorage` with its
`StorageNamespace` and `StateSchema`. The host supplies a
`MigrationDescriptor` with the declared source range and target schema.
`StateMigration::stage` checks the source version before running the migrator.
It passes the old state by immutable reference and stores the resulting
bytes separately for trial activation.

Pass `plan.candidate()` to the candidate version and commit the plan only once
the trial succeeds. Dropping the plan leaves the original snapshot in place.
Clone the original snapshot before staging if the host needs an independent
backup. Use `plan.commit_to(&store)` for previously persisted state. The
in-memory state stays intact if the persisted snapshot changed during the
trial or the adapter reports an error.

### Switching an active plugin to a new version

After package checks and state migration, call
`PluginManager::stage_handover(id, candidate)`. It initializes the candidate
and validates its registrations against host routes and other plugins.
An update must preserve the currently selected plugin route when it belongs
to that plugin. The router keeps the existing history.

Render and confirm the first frame using `handover.trial()`. Once it succeeds,
publish the staged package if applicable, then call `handover.commit()` to
deactivate the previous instance. An error from this final call reports
old-instance teardown. The confirmed candidate remains active.

On a failed trial, call `rollback()` or drop the handover to restore the old
instance and routes. Explicit rollback also reports candidate teardown errors.
If the host already presented the trial frame, render the restored view.
The handover manages plugin registrations and instances. The host owns
state migration, frame presentation, and package publication.

## Headless

`obscur_kit::headless::HeadlessHost` drives a real `UiRuntime` with no display —
the same frame path a window takes, which is what makes it usable as an oracle
rather than as a mock. `KitBootstrap::headless_host()` builds one.

Used by the spec test suite. See [11 — Diagnostics](11-diagnostics.md) for the
record/replay tier that sits beside it (**unwired** — no shipped host installs it).

## Choosing

Take `App` unless you need to own the window. The kit's own examples do both:
`hello_app.rs`, `file_manager.rs` and `calculator.rs` are `App`; `uikit_window.rs`,
`showcase_window.rs` and `controls_window.rs` spell out their own event loop and
are the reference for embedding.
