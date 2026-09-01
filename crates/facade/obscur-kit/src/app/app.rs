#![doc = include_str!("app.doc/_module.md")]

use super::app_error::AppError;
use super::app_parts::{AppParts, DiagnosticSinkBox, RowBuilder, RowClick, RowHold, ScrollHandler};
#[cfg(feature = "shell-layer")]
use super::layer_control::LayerControl;
#[cfg(feature = "window-winit")]
use super::runner;
use crate::bootstrap::{FontFaceBytes, FontSet};
use crate::control::AppControl;
use crate::geometry::Size;
use crate::headless::input::SelectorTarget;
use crate::identity::InternedSymbol;
use crate::interaction::event::ListenerPhase;
use crate::runtime::{CommandHandler, IntoCommandHandler};
use crate::scene::command::CanvasRegion;
use crate::screen::{OkuiScreen, ScreenData};
use crate::view::view::{CanvasPaint, View, ViewNode};
#[cfg(feature = "shell-layer")]
use crate::windowing::layer::LayerSettings;

#[doc = include_str!("app.doc/DEFAULT_SIZE.md")]
const DEFAULT_SIZE: (f32, f32) = (800.0, 600.0);

#[doc = include_str!("app.doc/DEFAULT_APP_ID.md")]
const DEFAULT_APP_ID: &str = "obscur-kit";

#[doc = include_str!("app.doc/FrameInfo.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameInfo {
    #[doc = include_str!("app.doc/viewport.md")]
    pub viewport: Size,
    #[doc = include_str!("app.doc/scale.md")]
    pub scale: f32,
}

#[doc = include_str!("app.doc/Root.md")]
enum Root {
    #[doc = include_str!("app.doc/View.md")]
    View(Box<dyn FnMut() -> View + 'static>),
    #[doc = include_str!("app.doc/Screen.md")]
    Screen(String),
    #[doc = include_str!("app.doc/ViewWith.md")]
    ViewWith(Box<dyn FnMut(&FrameInfo) -> View + 'static>),
}

#[doc = include_str!("app.doc/App.md")]
#[must_use = "an `App` does nothing until `.run()` is called"]
pub struct App {
    title: String,
    app_id: String,
    size: (f32, f32),
    stylesheets: Vec<String>,
    #[doc = include_str!("app.doc/fonts_field.md")]
    fonts: FontSet,
    root: Option<Root>,
    commands: Vec<(InternedSymbol, CommandHandler)>,
    #[doc = include_str!("app.doc/control.md")]
    control: AppControl,
    #[doc = include_str!("app.doc/screen_data.md")]
    screen_data: ScreenData,
    scroll: Option<ScrollHandler>,
    virtual_lists: Vec<(InternedSymbol, RowBuilder)>,
    virtual_clicks: Vec<(InternedSymbol, RowClick)>,
    virtual_holds: Vec<(InternedSymbol, RowHold)>,
    #[doc = include_str!("app.doc/diagnostics_field.md")]
    diagnostics: Option<DiagnosticSinkBox>,
    #[doc = include_str!("app.doc/layer_field.md")]
    #[cfg(feature = "shell-layer")]
    layer: Option<LayerSettings>,
    #[doc = include_str!("app.doc/layer_control_field.md")]
    #[cfg(feature = "shell-layer")]
    layer_control: LayerControl,
    #[cfg(feature = "shell-layer")]
    layer_waker: Option<crate::shell_layer::surface::LayerWaker>,
}

impl App {
    #[doc = include_str!("app.doc/new.md")]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            app_id: DEFAULT_APP_ID.to_owned(),
            size: DEFAULT_SIZE,
            stylesheets: Vec::new(),
            fonts: FontSet::new(),
            root: None,
            commands: Vec::new(),
            control: AppControl::new(),
            screen_data: ScreenData::new(),
            scroll: None,
            virtual_lists: Vec::new(),
            virtual_clicks: Vec::new(),
            virtual_holds: Vec::new(),
            diagnostics: None,
            #[cfg(feature = "shell-layer")]
            layer: None,
            #[cfg(feature = "shell-layer")]
            layer_control: LayerControl::new(),
            #[cfg(feature = "shell-layer")]
            layer_waker: None,
        }
    }

    #[doc = include_str!("app.doc/layer.md")]
    #[cfg(feature = "shell-layer")]
    pub fn layer(mut self, settings: LayerSettings) -> Self {
        self.layer = Some(settings);
        self
    }

    #[doc = include_str!("app.doc/displays.md")]
    #[cfg(feature = "shell-layer")]
    pub fn displays() -> Result<Vec<crate::windowing::layer::LayerDisplay>, AppError> {
        Ok(crate::shell_layer::surface::LayerProbe::displays()?)
    }

    #[doc = include_str!("app.doc/layer_control.md")]
    #[cfg(feature = "shell-layer")]
    #[must_use]
    pub fn layer_control(&self) -> LayerControl {
        self.layer_control.clone()
    }

    #[doc = include_str!("app.doc/waker.md")]
    #[cfg(feature = "shell-layer")]
    pub fn waker(mut self, waker: crate::shell_layer::surface::LayerWaker) -> Self {
        self.layer_waker = Some(waker);
        self
    }

    #[doc = include_str!("app.doc/app_id.md")]
    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = app_id.into();
        self
    }

    #[doc = include_str!("app.doc/with_size.md")]
    pub const fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }

    #[doc = include_str!("app.doc/stylesheet.md")]
    pub fn stylesheet(mut self, css: &str) -> Self {
        self.stylesheets.push(css.to_owned());
        self
    }

    #[doc = include_str!("app.doc/font.md")]
    pub fn font(mut self, family: &str, bytes: impl Into<std::sync::Arc<[u8]>>) -> Self {
        self.fonts.push(FontFaceBytes::new(
            crate::text::font::FontFamily::named(family),
            crate::text::font::FontAttributes::new(),
            bytes,
        ));
        self
    }

    #[doc = include_str!("app.doc/font_face.md")]
    pub fn font_face(mut self, face: FontFaceBytes) -> Self {
        self.fonts.push(face);
        self
    }

    #[doc = include_str!("app.doc/view.md")]
    pub fn view(mut self, builder: impl Fn() -> View + 'static) -> Self {
        self.root = Some(Root::View(Box::new(builder)));
        self
    }

    #[doc = include_str!("app.doc/view_with.md")]
    pub fn view_with(mut self, builder: impl FnMut(&FrameInfo) -> View + 'static) -> Self {
        self.root = Some(Root::ViewWith(Box::new(builder)));
        self
    }

    #[doc = include_str!("app.doc/virtual_list.md")]
    pub fn virtual_list(
        mut self,
        id: InternedSymbol,
        build: impl FnMut(usize) -> ViewNode + 'static,
    ) -> Self {
        self.virtual_lists.push((id, Box::new(build)));
        self
    }

    #[doc = include_str!("app.doc/on_row_click.md")]
    pub fn on_row_click(
        mut self,
        id: InternedSymbol,
        handler: impl FnMut(usize, u32) + 'static,
    ) -> Self {
        self.virtual_clicks.push((id, Box::new(handler)));
        self
    }

    #[doc = include_str!("app.doc/on_row_long_press.md")]
    pub fn on_row_long_press(
        mut self,
        id: InternedSymbol,
        handler: impl FnMut(usize) + 'static,
    ) -> Self {
        self.virtual_holds.push((id, Box::new(handler)));
        self
    }

    #[doc = include_str!("app.doc/on_scroll.md")]
    pub fn on_scroll(mut self, handler: impl FnMut(f32) + 'static) -> Self {
        self.scroll = Some(Box::new(handler));
        self
    }

    #[doc = include_str!("app.doc/painter.md")]
    pub fn painter(
        self,
        name: impl Into<String>,
        painter: impl FnMut(&mut CanvasRegion<'_>, &CanvasPaint<'_>) -> bool + 'static,
    ) -> Self {
        let name = name.into();
        // Queued on the control handle rather than kept in a field of its own:
        // the runner applies that queue after the built-ins are installed, so an
        // application painter under a built-in's key wins — which is the order a
        // host would expect and the reverse of what a field applied at
        // construction would give.
        self.control
            .with_runtime(move |runtime| runtime.register_named_canvas(&name, Box::new(painter)));
        self
    }

    #[doc = include_str!("app.doc/painter_id.md")]
    pub fn painter_id(
        self,
        id: InternedSymbol,
        painter: impl FnMut(&mut CanvasRegion<'_>, &CanvasPaint<'_>) -> bool + 'static,
    ) -> Self {
        self.control
            .with_runtime(move |runtime| runtime.register_canvas(id, Box::new(painter)));
        self
    }

    #[doc = include_str!("app.doc/screen.md")]
    pub fn screen(mut self, okui_src: &str) -> Self {
        self.root = Some(Root::Screen(okui_src.to_owned()));
        self
    }

    #[doc = include_str!("app.doc/screen_data_2.md")]
    #[must_use]
    pub fn screen_data(&self) -> ScreenData {
        self.screen_data.clone()
    }

    #[doc = include_str!("app.doc/control_2.md")]
    #[must_use]
    pub fn control(&self) -> AppControl {
        self.control.clone()
    }

    #[doc = include_str!("app.doc/diagnostics.md")]
    pub fn diagnostics(mut self, sink: impl obscur_diagnostics::DiagnosticSink + 'static) -> Self {
        self.diagnostics = Some(Box::new(sink));
        self
    }

    #[doc = include_str!("app.doc/on_command.md")]
    pub fn on_command<M>(
        mut self,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) -> Self {
        self.commands
            .push((command, handler.into_command_handler()));
        self
    }

    #[doc = include_str!("app.doc/on_command_at.md")]
    pub fn on_command_at<M>(
        self,
        target: SelectorTarget,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) -> Self {
        self.control.on_command_at(target, command, handler);
        self
    }

    #[doc = include_str!("app.doc/add_event_listener.md")]
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

    #[doc = include_str!("app.doc/run.md")]
    pub fn run(mut self) -> Result<(), AppError> {
        let root = self.root.take();
        #[cfg(feature = "shell-layer")]
        let layer = self.layer.take();
        #[cfg(feature = "shell-layer")]
        let layer_control = self.layer_control.clone();
        #[cfg(feature = "shell-layer")]
        let layer_waker = self.layer_waker.take();
        let screen_data = self.screen_data.clone();

        // Resolve the root into a `FnMut` builder the runtime can hold. A `.screen` is
        // compiled to IR here — once, before any surface is opened, so a compile
        // failure is an error rather than a blank window — and the builder lowers that
        // IR every frame, which is what lets `ScreenData` reach it.
        let builder: Box<dyn FnMut() -> View + 'static> = match root {
            Some(Root::View(builder)) => builder,
            Some(Root::Screen(src)) => {
                // Compiled against the table, so the names the application bound
                // before `run` are declared to the compiler and a reference to
                // anything else is a §2.13 diagnostic here rather than a value that
                // never arrives.
                let screen = OkuiScreen::compile_with(&src, screen_data)?;
                Box::new(move || screen.view())
            }
            // The frame-aware builder is wrapped by the windowed runner, which owns
            // the window and therefore the only authoritative viewport.
            Some(Root::ViewWith(builder)) => {
                #[cfg(feature = "shell-layer")]
                if layer.is_some() {
                    // A layer surface's viewport is the compositor's to give and it
                    // arrives before the first frame, so there is nothing a
                    // frame-aware builder would learn that `.view` does not.
                    return Err(AppError::FrameAwareLayerRoot);
                }
                #[cfg(feature = "window-winit")]
                return runner::run_with_frame(self.into_parts(), builder);
                #[cfg(not(feature = "window-winit"))]
                {
                    drop(builder);
                    return Err(AppError::NoWindowBackend);
                }
            }
            None => Box::new(View::default),
        };

        #[cfg(feature = "shell-layer")]
        if let Some(settings) = layer {
            return super::layer_runner::run(
                self.into_parts(),
                &settings,
                builder,
                &layer_control,
                layer_waker.as_ref(),
            );
        }

        #[cfg(feature = "window-winit")]
        {
            runner::run(self.into_parts(), builder)
        }
        #[cfg(not(feature = "window-winit"))]
        {
            drop(builder);
            Err(AppError::NoWindowBackend)
        }
    }

    #[doc = include_str!("app.doc/into_parts.md")]
    fn into_parts(self) -> AppParts {
        AppParts {
            title: self.title,
            app_id: self.app_id,
            size: self.size,
            stylesheets: self.stylesheets,
            fonts: self.fonts,
            commands: self.commands,
            control: self.control,
            scroll: self.scroll,
            virtual_lists: self.virtual_lists,
            virtual_clicks: self.virtual_clicks,
            virtual_holds: self.virtual_holds,
            diagnostics: self.diagnostics,
        }
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

    /// Audit 2-F: a face named on the builder has to survive as far as the
    /// `KitBootstrap` the runner installs the text stack from, or an
    /// application still cannot name a font. The runner's own line —
    /// `boot.font_set()` — needs a window and is not exercised here.
    #[test]
    fn an_application_face_reaches_the_bootstrap_that_installs_it() {
        use super::View;
        use crate::bootstrap::FontFaceBytes;
        use crate::text::font::{FontAttributes, FontFamily};
        use crate::text::span::FontWeight;

        let parts = App::new("t")
            .font("Figtree", &b"regular"[..])
            .font_face(FontFaceBytes::new(
                FontFamily::named("Figtree"),
                FontAttributes::new().with_weight(FontWeight::BOLD),
                &b"bold"[..],
            ))
            .into_parts();
        assert_eq!(parts.fonts.len(), 2, "both cuts travel with the parts");

        let booted = parts.boot(Box::new(View::default), super::Size::new(100.0, 100.0));
        let set = booted.boot.font_set();
        let named: Vec<&str> = set
            .faces()
            .iter()
            .map(|face| face.family().name())
            .collect();
        assert_eq!(
            named.iter().filter(|name| **name == "Figtree").count(),
            2,
            "the set the runner installs carries the application's faces: {named:?}"
        );
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
