//! [`App`] — the declarative builder for a standalone windowed application.

use super::app_error::AppError;
use super::runner;
use crate::control::AppControl;
use crate::geometry::Size;
use crate::headless::input::SelectorTarget;
use crate::identity::InternedSymbol;
use crate::interaction::event::ListenerPhase;
use crate::runtime::{CommandHandler, IntoCommandHandler};
use crate::screen::{OkuiScreen, ScreenData};
use crate::view::view::{View, ViewNode};

/// The default logical window size, in CSS pixels.
const DEFAULT_SIZE: (f32, f32) = (800.0, 600.0);

/// The default Wayland `app_id` / X11 `WM_CLASS` when the app sets none.
const DEFAULT_APP_ID: &str = "obscur-kit";

/// What the runtime knows about the frame that an app needs in order to size or
/// window its own content.
///
/// A view builder is otherwise blind: it cannot tell how tall the window is, so it
/// cannot decide how many rows are worth building. That is the whole prerequisite
/// for app-side virtualization — a list of 10,000 entries costs style, layout and
/// compose per node, so the only way to stay cheap is to not build the nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameInfo {
    /// The viewport in logical (CSS) pixels.
    pub viewport: Size,
    /// The display's device-pixel ratio.
    pub scale: f32,
}

/// The root the window renders. The last of [`App::view`] / [`App::view_with`] /
/// [`App::screen`] to be called wins.
enum Root {
    /// A reactive builder re-invoked each frame — the same shape
    /// [`UiRuntime::set_root_view`](crate::runtime::UiRuntime::set_root_view) holds.
    View(Box<dyn FnMut() -> View + 'static>),
    /// An `.okui` source, compiled to UI IR when [`App::run`] is called (a compile
    /// failure surfaces as an [`AppError`] from `run`) and lowered to a `View` on
    /// every frame through [`OkuiScreen`].
    Screen(String),
    /// A builder that additionally reads the frame's [`FrameInfo`].
    ViewWith(Box<dyn FnMut(&FrameInfo) -> View + 'static>),
}

/// A declarative, one-call builder for a **standalone windowed** application
/// (spec §1.4). Configure the window, styling, root view, and command handlers,
/// then [`run`](Self::run) to open the window and drive the event loop until it
/// closes.
///
/// See the [module documentation](super) for what `run` turns on by default.
#[must_use = "an `App` does nothing until `.run()` is called"]
pub struct App {
    title: String,
    app_id: String,
    size: (f32, f32),
    stylesheets: Vec<String>,
    root: Option<Root>,
    commands: Vec<(InternedSymbol, CommandHandler)>,
    /// The handle positional registrations, stylesheet replacements and queued
    /// runtime actions travel on; clones of it are handed to the application by
    /// [`App::control`].
    control: AppControl,
    /// The values an `.okui` [`screen`](App::screen)'s `{bindings}` read.
    screen_data: ScreenData,
    scroll: Option<Box<dyn FnMut(f32) + 'static>>,
    virtual_lists: Vec<(InternedSymbol, Box<dyn FnMut(usize) -> ViewNode + 'static>)>,
    virtual_clicks: Vec<(InternedSymbol, Box<dyn FnMut(usize, u32) + 'static>)>,
    virtual_holds: Vec<(InternedSymbol, Box<dyn FnMut(usize) + 'static>)>,
}

impl App {
    /// Starts a new application whose window carries `title`, at the default
    /// 800×600 logical size and with no styling or root yet.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            app_id: DEFAULT_APP_ID.to_owned(),
            size: DEFAULT_SIZE,
            stylesheets: Vec::new(),
            root: None,
            commands: Vec::new(),
            control: AppControl::new(),
            screen_data: ScreenData::new(),
            scroll: None,
            virtual_lists: Vec::new(),
            virtual_clicks: Vec::new(),
            virtual_holds: Vec::new(),
        }
    }

    /// Sets the Wayland `app_id` (and X11 `WM_CLASS`) the window reports, defaulting
    /// to `"obscur-kit"`.
    ///
    /// This is the identity a desktop environment matches on — a compositor's rules
    /// (Hyprland's `class`, sway's `app_id`), the taskbar, and `.desktop` file
    /// association all key off it, **not** off the human-readable
    /// [`title`](Self::new). Give each application a distinct one so a rule can
    /// target it alone.
    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = app_id.into();
        self
    }

    /// Sets the initial window size in **logical** (CSS) pixels. The window stays
    /// resizable; a resize reflows the layout at the display's HiDPI density.
    pub const fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }

    /// Appends an authored stylesheet. Multiple calls accumulate in order; all are
    /// installed (below the UA default sheet) through the motion-aware
    /// [`CssPipeline`](crate::bootstrap::CssPipeline), so `@keyframes`/`transition`
    /// work and CSS errors reach the diagnostics channel.
    ///
    /// This is the *initial* set. To replace it while the application runs — a theme
    /// switch, a re-read from disk — call
    /// [`AppControl::set_stylesheets`](crate::control::AppControl::set_stylesheets) on
    /// a [`control`](Self::control) handle: §4.12's explicit invalidation, which used
    /// to be unreachable because the sheets were collected once and nothing ever
    /// re-supplied them.
    pub fn stylesheet(mut self, css: &str) -> Self {
        self.stylesheets.push(css.to_owned());
        self
    }

    /// Sets the root to a reactive Rust view `builder`, re-invoked each reactive
    /// frame — exactly the shape
    /// [`UiRuntime::set_root_view`](crate::runtime::UiRuntime::set_root_view) takes.
    /// Overrides any previous [`view`](Self::view) / [`screen`](Self::screen).
    pub fn view(mut self, builder: impl Fn() -> View + 'static) -> Self {
        self.root = Some(Root::View(Box::new(builder)));
        self
    }

    /// Sets the root to a builder that also receives the frame's [`FrameInfo`] —
    /// the viewport and the device scale.
    ///
    /// Use this when what to build depends on how much is visible: a long list
    /// should window itself to the rows the viewport can show rather than emit one
    /// node per item. Overrides any previous [`view`](Self::view) /
    /// [`screen`](Self::screen).
    pub fn view_with(mut self, builder: impl FnMut(&FrameInfo) -> View + 'static) -> Self {
        self.root = Some(Root::ViewWith(Box::new(builder)));
        self
    }

    /// Registers the row builder a virtual list realizes through (spec §4.9).
    ///
    /// The view names this list with `ViewNode::virtual_list(id, count, row_extent)`
    /// and states how many items exist; `build(index)` produces one row and is
    /// called only for the indices the viewport can show. The runtime owns the
    /// window, the scroll offset and the content extent — which is the half an app
    /// cannot own for itself, because stating the full height would mean building
    /// the full list.
    pub fn virtual_list(
        mut self,
        id: InternedSymbol,
        build: impl FnMut(usize) -> ViewNode + 'static,
    ) -> Self {
        self.virtual_lists.push((id, Box::new(build)));
        self
    }

    /// Registers a click handler for the rows of virtual list `id` (spec §4.9).
    ///
    /// When a row is clicked, `handler(index, clicks)` runs with the entry's **model
    /// index** — its place in the full collection, not among the built nodes — and the
    /// click count (`1` single, `2` double), so an app can select on a single click
    /// and open on a double. This is the piece that makes a virtualized list an
    /// *interactive* list: a per-row `on:click` cannot, because every row a builder
    /// emits shares one command symbol, so only the runtime's geometry can say which
    /// row was hit.
    ///
    /// # Still outside §2.3's model
    ///
    /// Now that [`on_command`](Self::on_command) delivers the event, "a handler is
    /// given nothing" is no longer the reason this signature exists — the *model
    /// index* is. But the payload does not reach here: the runtime calls a virtual-row
    /// handler directly from its pointer path rather than dispatching a command
    /// through §2.3's capture → target → bubble walk, so a row click carries no
    /// [`UiEvent`](crate::interaction::event::UiEvent), no listener on an ancestor can
    /// observe it, and `prevent_default` cannot suppress it. An application that needs
    /// the pointer's position, button or device on a row still cannot have it.
    pub fn on_row_click(
        mut self,
        id: InternedSymbol,
        handler: impl FnMut(usize, u32) + 'static,
    ) -> Self {
        self.virtual_clicks.push((id, Box::new(handler)));
        self
    }

    /// Registers a **long-press** handler for the rows of virtual list `id`: a
    /// finger held still on a row calls `handler(model_index)` while it is still
    /// down, and the lift that follows is not a click.
    ///
    /// The touch counterpart to [`on_row_click`](Self::on_row_click), and the only
    /// way a virtualized row reaches a context action: a touchscreen has one
    /// button and no modifiers, so a hold is what a secondary click would have
    /// been — and a realized row cannot carry a per-row binding to receive it.
    ///
    /// Outside §2.3's propagation for the same reason as
    /// [`on_row_click`](Self::on_row_click): the hold is recognised on the runtime's
    /// pointer path and calls this directly, so no event is delivered and no listener
    /// sees it.
    pub fn on_row_long_press(
        mut self,
        id: InternedSymbol,
        handler: impl FnMut(usize) + 'static,
    ) -> Self {
        self.virtual_holds.push((id, Box::new(handler)));
        self
    }

    /// Registers a handler for wheel scrolling the kit did not consume.
    ///
    /// A `overflow: scroll` container inside the kit takes the wheel first; this is
    /// what is left over, in logical pixels, positive when the content should move
    /// down. An app that windows its own list owns its scroll position and needs
    /// this to move it.
    ///
    /// Not a §2.3 event either: a wheel notch no container consumed is offered to this
    /// closure by the window runner, not dispatched as a command, so there is nothing
    /// to observe or prevent. It stays an `FnMut(f32)` because that is the whole
    /// payload it has — leftover pixels along the block axis.
    pub fn on_scroll(mut self, handler: impl FnMut(f32) + 'static) -> Self {
        self.scroll = Some(Box::new(handler));
        self
    }

    /// Sets the root to an `.okui` screen: parsed and compiled to UI IR when
    /// [`run`](Self::run) is called, then lowered to a [`View`] on **every frame**,
    /// with `{bindings}` resolved against [`screen_data`](Self::screen_data). A
    /// compile failure makes `run` return an [`AppError`] carrying the diagnostics.
    /// Overrides any previous [`view`](Self::view) / [`screen`](Self::screen).
    ///
    /// # What state can and cannot reach it
    ///
    /// This used to materialise the screen **once** and hand the runtime a builder
    /// that returned that one `View` for the application's whole life, so no state
    /// change could reach it at all. Now the compiled IR is kept and re-lowered per
    /// frame (nothing re-parses), which means:
    ///
    /// - **values do reach it**: a body `{count}` and a bound attribute
    ///   (`class={theme}`) read [`ScreenData`], so writing that table between frames
    ///   changes what the screen shows. A name an *attribute* binds must be bound in
    ///   [`screen_data`](Self::screen_data) **before** `run`, because that is what
    ///   declares it to the compiler — an undeclared name is a §2.13 error from `run`
    ///   rather than a value that silently never arrives;
    /// - **shape does not**: `<If>`/`<For>` are statically materialised by the
    ///   bridge on this route, so the *structure* is fixed at compile time, and a
    ///   `{binding}` inside a `<VirtualFor>` keeps its placeholder (per-row data
    ///   comes from [`virtual_list`](Self::virtual_list), which is told the row's
    ///   model index);
    /// - **contracts are validated here, not at compile time**: §2.9 asks for
    ///   compile-time contract validation, and a source that is a string until `run`
    ///   cannot have it. Use the **build-time** route for that — a `build.rs` that
    ///   emits through
    ///   [`CodegenRoot::facade`](crate::okui_compiler::codegen::CodegenRoot::facade),
    ///   or the inline `okui!` macro — where the markup becomes ordinary Rust,
    ///   `{count}` reads the application's own binding, and a contract error is a
    ///   compile error. Pass the generated function to [`view`](Self::view).
    ///
    /// See [`OkuiScreen`] for the whole comparison between the two routes.
    pub fn screen(mut self, okui_src: &str) -> Self {
        self.root = Some(Root::Screen(okui_src.to_owned()));
        self
    }

    /// The table an `.okui` [`screen`](Self::screen)'s `{bindings}` read, as a handle
    /// application code keeps.
    ///
    /// Write it before `run` for the initial values and from a command handler for
    /// every change afterwards; the next frame's lowering reads what is there. It is
    /// plain application state — no runtime is involved — so a write is immediate and
    /// needs no deferral, unlike [`control`](Self::control)'s requests.
    ///
    /// Returns a clone of one shared table, so several handlers may each hold one.
    #[must_use]
    pub fn screen_data(&self) -> ScreenData {
        self.screen_data.clone()
    }

    /// The handle onto this application's own running runtime: positional command
    /// handlers, §2.3 listeners, stylesheet invalidation, and a queued
    /// [`with_runtime`](AppControl::with_runtime) escape hatch.
    ///
    /// Returns a clone of the one handle the runner applies, so a command handler may
    /// capture it. Read [`AppControl`] for **when** a queued request takes effect — at
    /// the next frame boundary, never inside the handler that asked — and for why
    /// nothing hands out a `&mut UiRuntime`.
    #[must_use]
    pub fn control(&self) -> AppControl {
        self.control.clone()
    }

    /// Registers `handler` for `command` at the **window** level — link 4 of §2.6's
    /// routing chain, and the right level for a command that means the same thing
    /// anywhere in this window.
    ///
    /// # Two accepted shapes (spec §2.3)
    ///
    /// ```ignore
    /// .on_command(SAVE, || save())                          // takes nothing
    /// .on_command(PICK, |e: &mut UiEvent| pick(e.target()))  // takes the event
    /// ```
    ///
    /// The event form is how §2.3's model reaches an application: `e.target()` is the
    /// node that was hit, `e.current_target()` where the binding resolved,
    /// `e.pointer()` / `e.key()` §2.4's payload, `e.trust()` the trusted/untrusted
    /// origin, and `e.stop_propagation()` / `e.prevent_default()` the steering. Until
    /// this widened, the handler was an `FnMut()` given nothing, so none of it was
    /// reachable and a handler had to *capture* everything it might need.
    ///
    /// A one-argument closure needs its parameter annotated (`|e: &mut UiEvent|`) —
    /// inference cannot pick the marker and the closure's signature at once. The
    /// zero-argument form is unchanged and is still right for most handlers.
    ///
    /// For a handler that must mean different things in different places, register it
    /// at the node instead: [`on_command_at`](Self::on_command_at).
    pub fn on_command<M>(
        mut self,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) -> Self {
        self.commands
            .push((command, handler.into_command_handler()));
        self
    }

    /// Registers `handler` for `command` **at the node** `target` names — links 1 and
    /// 3 of §2.6's routing chain, where the same symbol runs different logic
    /// depending on where the command came from.
    ///
    /// `target` is a §7.5 stable identifier: write `test-id="save"` on the element
    /// (in `.okui` markup, or as a `TEST_ID` property in a Rust view) and name it
    /// here with [`SelectorTarget::test_id`]. A [`NodeId`](crate::identity::NodeId)
    /// works too, for a node an application already read back from the tree — but a
    /// name is what an application configuring itself before `run` can have, since no
    /// node id exists until a frame has built the tree.
    ///
    /// The name is resolved after the first commit, and resolved again whenever the
    /// element it was bound to stops carrying it — so the registration follows the
    /// **name**, across a rebuilt subtree. A name that matches nothing stays unbound and
    /// never fires; ask [`AppControl::bound_node`] instead of guessing.
    ///
    /// A handler here **claims** the command and stops the chain: no ancestor, no
    /// window handler ([`on_command`](Self::on_command)) and no application fallback
    /// sees it. Both handler shapes are accepted, as in `on_command`.
    pub fn on_command_at<M>(
        self,
        target: SelectorTarget,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) -> Self {
        self.control.on_command_at(target, command, handler);
        self
    }

    /// Registers a §2.3 **listener** for `command` on the node `target` names, running
    /// in `phase`.
    ///
    /// A listener observes and steers a command as it propagates, where a handler
    /// claims it. With [`ListenerPhase::Capture`] it sees a command aimed at a
    /// *descendant* before the target does — an overlay or a gesture arbiter taking
    /// the interaction away — and it can call
    /// [`stop_propagation`](crate::interaction::event::UiEvent::stop_propagation) to
    /// end the propagation or
    /// [`prevent_default`](crate::interaction::event::UiEvent::prevent_default) to let
    /// it continue with the command's handler suppressed. Several listeners per node
    /// are allowed and run in registration order.
    ///
    /// Same targeting rules as [`on_command_at`](Self::on_command_at).
    pub fn add_event_listener<M>(
        self,
        target: SelectorTarget,
        command: InternedSymbol,
        phase: ListenerPhase,
        listener: impl IntoCommandHandler<M>,
    ) -> Self {
        self.control
            .add_event_listener(target, command, phase, listener);
        self
    }

    /// Opens a window, wires the full standalone stack, and runs the event loop,
    /// **blocking** until the window is closed.
    ///
    /// # Errors
    ///
    /// Returns an [`AppError`] if an `.okui` [`screen`](Self::screen) fails to
    /// compile, the event loop cannot be built, the window cannot be created, or no
    /// usable GPU adapter/device/surface is available.
    pub fn run(mut self) -> Result<(), AppError> {
        // Resolve the root into a `FnMut` builder the runtime can hold. A `.screen` is
        // compiled to IR here — once, before any window is opened, so a compile
        // failure is an error rather than a blank window — and the builder lowers that
        // IR every frame, which is what lets `ScreenData` reach it.
        let builder: Box<dyn FnMut() -> View + 'static> = match self.root.take() {
            Some(Root::View(builder)) => builder,
            Some(Root::Screen(src)) => {
                // Compiled against the table, so the names the application bound
                // before `run` are declared to the compiler and a reference to
                // anything else is a §2.13 diagnostic here rather than a value that
                // never arrives.
                let screen = OkuiScreen::compile_with(&src, self.screen_data.clone())?;
                Box::new(move || screen.view())
            }
            // The frame-aware builder is wrapped by the runner, which owns the
            // window and therefore the only authoritative viewport.
            Some(Root::ViewWith(builder)) => {
                return runner::run_with_frame(
                    self.title,
                    self.app_id,
                    self.size,
                    self.stylesheets,
                    builder,
                    self.commands,
                    self.control,
                    self.scroll,
                    self.virtual_lists,
                    self.virtual_clicks,
                    self.virtual_holds,
                );
            }
            None => Box::new(View::default),
        };
        runner::run(
            self.title,
            self.app_id,
            self.size,
            self.stylesheets,
            builder,
            self.commands,
            self.control,
            self.scroll,
            self.virtual_lists,
            self.virtual_clicks,
            self.virtual_holds,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{App, Root, ScreenData, View};
    use crate::headless::input::SelectorTarget;
    use crate::identity::InternedSymbol;
    use crate::interaction::event::{ListenerPhase, UiEvent};

    /// A command symbol the builder cases below register against.
    const PICK: InternedSymbol = InternedSymbol::from_raw(0x5A01);

    #[test]
    fn stylesheets_accumulate_in_order() {
        let app = App::new("t")
            .stylesheet(".a {}")
            .stylesheet(".b {}")
            .stylesheet(".c {}");
        assert_eq!(app.stylesheets, vec![".a {}", ".b {}", ".c {}"]);
    }

    #[test]
    fn screen_then_view_lets_the_last_root_win() {
        // Both set the root; the last call wins, so a later `.view` clears a screen.
        let app = App::new("t").screen("<div/>").view(View::default);
        assert!(matches!(app.root, Some(Root::View(_))));
    }

    #[test]
    fn both_handler_shapes_register_through_the_builder() {
        // The 1-C widening, at the builder: the zero-argument form still compiles and
        // the event-taking form now does too. Registration is what is checked here;
        // delivery is `tests/event_model_delivery.rs`.
        let app = App::new("t")
            .on_command(PICK, || {})
            .on_command(PICK, |_e: &mut UiEvent| {});
        assert_eq!(app.commands.len(), 2);
    }

    #[test]
    fn positional_registrations_travel_on_the_control_handle() {
        // They cannot be resolved yet — no frame has run — so what the builder does is
        // record them where the runner will bind them after the first commit.
        let app = App::new("t")
            .on_command_at(SelectorTarget::test_id("save"), PICK, || {})
            .add_event_listener(
                SelectorTarget::test_id("list"),
                PICK,
                ListenerPhase::Capture,
                |_e: &mut UiEvent| {},
            );
        let control = app.control();
        assert_eq!(control.binding_count(), 2);
        assert_eq!(
            control.bound_node(&SelectorTarget::test_id("save")),
            None,
            "a name resolves against a tree, and there is none before `run`"
        );
    }

    #[test]
    fn the_screen_data_handle_is_the_one_the_screen_will_read() {
        let app = App::new("t").screen("<div>{count}</div>");
        let data: ScreenData = app.screen_data();
        data.set("count", 7_i64);
        assert_eq!(app.screen_data.int("count"), 7, "one table, two handles");
    }
}
