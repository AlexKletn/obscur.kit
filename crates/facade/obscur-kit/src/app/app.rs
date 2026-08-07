#![doc = include_str!("app.doc/_module.md")]

use super::app_error::AppError;
use super::runner;
use crate::control::AppControl;
use crate::geometry::Size;
use crate::headless::input::SelectorTarget;
use crate::identity::InternedSymbol;
use crate::interaction::event::ListenerPhase;
use crate::runtime::{CommandHandler, IntoCommandHandler};
use crate::scene::command::CanvasRegion;
use crate::screen::{OkuiScreen, ScreenData};
use crate::view::view::{CanvasPaint, View, ViewNode};

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
    root: Option<Root>,
    commands: Vec<(InternedSymbol, CommandHandler)>,
    #[doc = include_str!("app.doc/control.md")]
    control: AppControl,
    #[doc = include_str!("app.doc/screen_data.md")]
    screen_data: ScreenData,
    scroll: Option<Box<dyn FnMut(f32) + 'static>>,
    virtual_lists: Vec<(InternedSymbol, Box<dyn FnMut(usize) -> ViewNode + 'static>)>,
    virtual_clicks: Vec<(InternedSymbol, Box<dyn FnMut(usize, u32) + 'static>)>,
    virtual_holds: Vec<(InternedSymbol, Box<dyn FnMut(usize) + 'static>)>,
}

impl App {
    #[doc = include_str!("app.doc/new.md")]
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
