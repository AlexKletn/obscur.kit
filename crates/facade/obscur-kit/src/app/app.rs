//! [`App`] — the declarative builder for a standalone windowed application.

use super::app_error::AppError;
use super::runner;
use crate::geometry::Size;
use crate::identity::InternedSymbol;
use crate::okui_compiler::bridge::UiIrViewBridge;
use crate::okui_compiler::compile::Compiler;
use crate::okui_syntax::ast::Document;
use crate::okui_syntax::parser::Parser;
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
    /// An `.okui` source, compiled to a `View` when [`App::run`] is called (a
    /// compile failure surfaces as an [`AppError`] from `run`).
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
    commands: Vec<(InternedSymbol, Box<dyn FnMut() + 'static>)>,
    scroll: Option<Box<dyn FnMut(f32) + 'static>>,
    virtual_lists: Vec<(InternedSymbol, Box<dyn FnMut(usize) -> ViewNode + 'static>)>,
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
            scroll: None,
            virtual_lists: Vec::new(),
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

    /// Registers a handler for wheel scrolling the kit did not consume.
    ///
    /// A `overflow: scroll` container inside the kit takes the wheel first; this is
    /// what is left over, in logical pixels, positive when the content should move
    /// down. An app that windows its own list owns its scroll position and needs
    /// this to move it.
    pub fn on_scroll(mut self, handler: impl FnMut(f32) + 'static) -> Self {
        self.scroll = Some(Box::new(handler));
        self
    }

    /// Sets the root to an `.okui` screen, compiled (parse → compile → bridge) to a
    /// static [`View`] when [`run`](Self::run) is called; a compile failure makes
    /// `run` return an [`AppError`] carrying the diagnostics. Overrides any previous
    /// [`view`](Self::view) / [`screen`](Self::screen).
    pub fn screen(mut self, okui_src: &str) -> Self {
        self.root = Some(Root::Screen(okui_src.to_owned()));
        self
    }

    /// Registers `handler` for `command`, forwarded verbatim into the runtime's
    /// [`on_command`](crate::runtime::UiRuntime::on_command) before the loop starts.
    /// The handler type is identical to the runtime's: an `FnMut()` that typically
    /// captures app state through an `Rc<RefCell<_>>` / `Rc<Cell<_>>`.
    pub fn on_command(mut self, command: InternedSymbol, handler: impl FnMut() + 'static) -> Self {
        self.commands.push((command, Box::new(handler)));
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
        // Resolve the root into a `FnMut` builder the runtime can hold. A `.screen`
        // is materialized once here (static v1 materialization); a compile failure
        // is surfaced before any window is opened.
        let builder: Box<dyn FnMut() -> View + 'static> = match self.root.take() {
            Some(Root::View(builder)) => builder,
            Some(Root::Screen(src)) => {
                let view = compile_screen(&src)?;
                Box::new(move || view.clone())
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
                    self.scroll,
                    self.virtual_lists,
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
            self.scroll,
            self.virtual_lists,
        )
    }
}

/// Compiles an `.okui` source string to a [`View`] through the real frontend
/// pipeline (parse → compile → [`UiIrViewBridge`]), mapping any failure to an
/// [`AppError`] that carries the diagnostics.
fn compile_screen(src: &str) -> Result<View, AppError> {
    let parsed = Parser::new().parse(src);
    if !parsed.errors().is_empty() {
        return Err(AppError::ScreenParse(
            parsed.errors().iter().map(ToString::to_string).collect(),
        ));
    }
    let document = Document::cast(parsed.tree().root().clone()).ok_or_else(|| {
        AppError::ScreenParse(vec!["the `.okui` root is not a document".to_owned()])
    })?;
    let result = Compiler::new("screen.okui").compile(&document);
    if result.has_errors() {
        return Err(AppError::ScreenCompile(
            result
                .diagnostics()
                .iter()
                .map(ToString::to_string)
                .collect(),
        ));
    }
    let document = result
        .document()
        .ok_or_else(|| AppError::ScreenCompile(vec!["no document was produced".to_owned()]))?;
    Ok(UiIrViewBridge::document_to_view(document))
}

#[cfg(test)]
mod tests {
    use super::{App, AppError, Root, View, compile_screen};

    #[test]
    fn a_valid_okui_screen_compiles_to_a_root_view() {
        // The exact frontend path `.screen` + `run` take, exercised without a loop.
        let view = compile_screen(r#"<div class="counter"><span>0</span></div>"#)
            .expect("a well-formed .okui screen compiles");
        assert_eq!(view.root().len(), 1, "one root element bridged");
    }

    #[test]
    fn an_invalid_okui_screen_surfaces_an_apperror_with_diagnostics() {
        // `<slot/>` without a `name` is a §2.12 compile error: it parses but the
        // compile reports it, so `.screen` must surface it as an `AppError`.
        let error = compile_screen("<div><slot/></div>")
            .expect_err("a malformed .okui screen is an error, not a silent empty view");
        match error {
            AppError::ScreenCompile(diagnostics) => {
                assert!(!diagnostics.is_empty(), "the diagnostics are carried");
            }
            other => panic!("expected a ScreenCompile error, got {other:?}"),
        }
    }

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
}
