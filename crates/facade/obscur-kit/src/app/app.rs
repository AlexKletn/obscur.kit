//! [`App`] — the declarative builder for a standalone windowed application.

mod app_root;

use app_root::AppRoot;

use super::app_error::AppError;
use super::app_parts::{AppParts, DiagnosticSinkBox, RowBuilder, RowClick, RowHold, ScrollHandler};
#[cfg(feature = "shell-layer")]
use super::layer_control::LayerControl;
#[cfg(feature = "window-winit")]
use super::runner;
use crate::bootstrap::{FontFaceBytes, FontSet};
use crate::control::AppControl;
#[cfg(test)]
use crate::geometry::Size;
use crate::headless::input::SelectorTarget;
use crate::identity::InternedSymbol;
use crate::interaction::event::ListenerPhase;
use crate::runtime::{CommandHandler, IntoCommandHandler};
use crate::scene::command::CanvasRegion;
use crate::screen::{OkuiScreen, ScreenData};
use crate::system::appearance::AppearanceProvider;
use crate::view::component::ComponentRegistration;
use crate::view::contract::ComponentSurface;
use crate::view::view::{CanvasPaint, View, ViewNode};
#[cfg(feature = "shell-layer")]
use crate::windowing::layer::LayerSettings;

/// The default logical window size, in CSS pixels.
const DEFAULT_SIZE: (f32, f32) = (800.0, 600.0);

/// The default Wayland `app_id` / X11 `WM_CLASS` when the app sets none.
const DEFAULT_APP_ID: &str = "obscur-kit";

use super::FrameInfo;

#[doc = include_str!("../../docs/api/app/app.doc/App.md")]
#[must_use = "an `App` does nothing until `.run()` is called"]
pub struct App {
    render_backend: Option<super::RenderBackend>,
    background_blur: bool,
    title: String,
    app_id: String,
    #[cfg(feature = "window-winit")]
    decorated: bool,
    #[cfg(feature = "window-winit")]
    resize: bool,
    #[cfg(feature = "window-winit")]
    window_mode: crate::WindowMode,
    #[cfg(feature = "window-winit")]
    prefer_opengl: bool,
    size: (f32, f32),
    #[cfg(feature = "window-winit")]
    size_limits: super::window_size_limits::WindowSizeLimits,
    #[cfg(feature = "window-winit")]
    window_waker: Option<super::app_parts::WindowWakerCallback>,
    stylesheets: Vec<String>,
    appearance_provider: Option<Box<dyn AppearanceProvider>>,
    #[doc = include_str!("../../docs/api/app/app.doc/fonts_field.md")]
    fonts: FontSet,
    root: Option<AppRoot>,
    commands: Vec<(InternedSymbol, CommandHandler)>,
    components: Vec<ComponentRegistration>,
    screen_contracts: Vec<ComponentSurface>,
    /// The handle positional registrations, stylesheet replacements and queued
    /// runtime actions travel on; clones of it are handed to the application by
    /// [`App::control`].
    control: AppControl,
    /// The values an `.okui` [`screen`](App::screen)'s `{bindings}` read.
    screen_data: ScreenData,
    scroll: Option<ScrollHandler>,
    virtual_lists: Vec<(InternedSymbol, RowBuilder)>,
    virtual_clicks: Vec<(InternedSymbol, RowClick)>,
    virtual_holds: Vec<(InternedSymbol, RowHold)>,
    /// Where this application's diagnostics go, if it said.
    diagnostics: Option<DiagnosticSinkBox>,
    #[doc = include_str!("../../docs/api/app/app.doc/layer_field.md")]
    #[cfg(feature = "shell-layer")]
    layer: Option<LayerSettings>,
    /// The live handle to the surface, made up front so a handler can hold a clone of
    /// it before [`run`](Self::run) takes ownership of everything else.
    #[cfg(feature = "shell-layer")]
    layer_control: LayerControl,
    #[cfg(feature = "shell-layer")]
    layer_waker: Option<crate::shell_layer::surface::LayerWaker>,
}

impl App {
    /// Starts a new application whose window carries `title`, at the default
    /// 800×600 logical size and with no styling or root yet.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            render_backend: None,
            background_blur: false,
            title: title.into(),
            app_id: DEFAULT_APP_ID.to_owned(),
            #[cfg(feature = "window-winit")]
            decorated: true,
            #[cfg(feature = "window-winit")]
            resize: true,
            #[cfg(feature = "window-winit")]
            window_mode: crate::WindowMode::Normal,
            #[cfg(feature = "window-winit")]
            prefer_opengl: false,
            size: DEFAULT_SIZE,
            #[cfg(feature = "window-winit")]
            size_limits: super::window_size_limits::WindowSizeLimits::default(),
            #[cfg(feature = "window-winit")]
            window_waker: None,
            stylesheets: Vec::new(),
            appearance_provider: None,
            fonts: FontSet::new(),
            root: None,
            commands: Vec::new(),
            components: Vec::new(),
            screen_contracts: Vec::new(),
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

    #[doc = include_str!("../../docs/api/app/app.doc/layer.md")]
    #[cfg(feature = "shell-layer")]
    pub fn layer(mut self, settings: LayerSettings) -> Self {
        self.layer = Some(settings);
        self
    }

    /// Queries the actual desktop panel adapter before creating a surface. Unsupported lets applications offer a window or in-window alternative.
    pub fn layer_capabilities()
    -> Result<crate::windowing::port::LayerCapabilities, crate::windowing::port::LayerError> {
        #[cfg(feature = "shell-layer")]
        {
            use crate::windowing::port::LayerPlatform;
            crate::shell_layer::surface::LayerProbe.capabilities()
        }
        #[cfg(not(feature = "shell-layer"))]
        Err(crate::windowing::port::LayerError::Unsupported)
    }

    /// Chooses a desktop panel when the adapter and requested keyboard input are available. Otherwise keeps an ordinary window. Discovery errors are returned to the application.
    #[cfg(feature = "window-winit")]
    pub fn layer_or_window(
        self,
        settings: crate::windowing::layer::LayerSettings,
    ) -> Result<Self, crate::windowing::port::LayerError> {
        #[cfg(feature = "shell-layer")]
        {
            use crate::windowing::layer::LayerFocusPolicy;
            use crate::windowing::port::{LayerError, LayerFeature};
            use obscur_capability::SupportLevel;

            let available = match Self::layer_capabilities() {
                Ok(capabilities) => {
                    let feature = match settings.focus {
                        LayerFocusPolicy::Never => None,
                        LayerFocusPolicy::OnDemand => Some(LayerFeature::OnDemandKeyboard),
                        LayerFocusPolicy::Exclusive => Some(LayerFeature::ExclusiveKeyboard),
                    };
                    feature.is_none_or(|feature| {
                        capabilities.supports(feature) != SupportLevel::Unavailable
                    })
                }
                Err(LayerError::Unsupported) => false,
                Err(error) => return Err(error),
            };
            if available {
                return Ok(self.layer(settings));
            }
            let mut app = self;
            app.layer = None;
            Ok(app)
        }
        #[cfg(not(feature = "shell-layer"))]
        {
            let _ = settings;
            Ok(self)
        }
    }

    #[doc = include_str!("../../docs/api/app/app.doc/displays.md")]
    #[cfg(feature = "shell-layer")]
    pub fn displays() -> Result<Vec<crate::windowing::layer::LayerDisplay>, AppError> {
        Ok(crate::shell_layer::surface::LayerProbe::displays()?)
    }

    #[doc = include_str!("../../docs/api/app/app.doc/layer_control.md")]
    #[cfg(feature = "shell-layer")]
    #[must_use]
    pub fn layer_control(&self) -> LayerControl {
        self.layer_control.clone()
    }

    #[doc = include_str!("../../docs/api/app/app.doc/waker.md")]
    #[cfg(feature = "shell-layer")]
    pub fn waker(mut self, waker: crate::shell_layer::surface::LayerWaker) -> Self {
        self.layer_waker = Some(waker);
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/app_id.md")]
    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = app_id.into();
        self
    }

    /// Controls whether the native window supplies its own frame and title bar.
    #[cfg(feature = "window-winit")]
    pub const fn with_decorations(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/with_resize.md")]
    #[cfg(feature = "window-winit")]
    pub const fn with_resize(mut self, resize: bool) -> Self {
        self.resize = resize;
        self
    }

    /// Prefer a hardware OpenGL adapter for the ordinary desktop window. If no usable
    /// hardware OpenGL adapter is found, adapter selection follows its normal order.
    /// This setting selects the graphics API underneath the renderer.
    #[cfg(feature = "window-winit")]
    pub const fn prefer_opengl(mut self, enabled: bool) -> Self {
        self.prefer_opengl = enabled;
        self
    }

    /// Supplies a thread-safe function that requests a new UI frame from a background worker.
    /// The function returns false once the window event loop has closed.
    #[cfg(feature = "window-winit")]
    pub fn on_window_waker(
        mut self,
        ready: impl FnOnce(Box<dyn Fn() -> bool + Send + Sync>) + 'static,
    ) -> Self {
        self.window_waker = Some(Box::new(ready));
        self
    }

    /// Requests compositor blur over the full surface for this application. The effect depends on Wayland support and is visible through translucent content.
    pub const fn with_background_blur(mut self, enabled: bool) -> Self {
        self.background_blur = enabled;
        self
    }

    /// Selects a GPU renderer for this application. An explicit selection takes precedence over `OBSCUR_RENDERER`. `run` returns an error when the selected feature is absent.
    pub const fn with_renderer(mut self, backend: super::RenderBackend) -> Self {
        self.render_backend = Some(backend);
        self
    }

    /// Restores a supported snapshot at startup. Current size limits still apply. Unsupported snapshots leave configuration unchanged.
    #[cfg(feature = "window-winit")]
    pub fn with_window_placement(mut self, placement: crate::WindowPlacement) -> Self {
        if placement.is_supported() {
            if let Some(size) = placement.normal_size() {
                self.size = size;
            }
            self.window_mode = placement.mode();
        }
        self
    }

    /// Set the initial window size in logical pixels. For desktop windows,
    /// `App::with_resize` controls whether the user can change that size.
    pub const fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }

    /// Set the minimum desktop window content width and height in logical pixels.
    /// Both dimensions must be finite and at least one. The initial size is clamped
    /// to the configured bounds. The platform applies the limits during resizing.
    #[cfg(feature = "window-winit")]
    pub const fn with_min_size(mut self, width: f32, height: f32) -> Self {
        self.size_limits.minimum = Some((width, height));
        self
    }

    /// Set the maximum desktop window content width and height in logical pixels.
    /// Both dimensions must be finite and at least one. Each maximum must be at
    /// least the corresponding configured minimum. The platform applies the limits
    /// during resizing.
    #[cfg(feature = "window-winit")]
    pub const fn with_max_size(mut self, width: f32, height: f32) -> Self {
        self.size_limits.maximum = Some((width, height));
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/stylesheet.md")]
    pub fn stylesheet(mut self, css: &str) -> Self {
        self.stylesheets.push(css.to_owned());
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/font.md")]
    pub fn font(mut self, family: &str, bytes: impl Into<std::sync::Arc<[u8]>>) -> Self {
        self.fonts.push(FontFaceBytes::new(
            crate::text::font::FontFamily::named(family),
            crate::text::font::FontAttributes::new(),
            bytes,
        ));
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/font_face.md")]
    pub fn font_face(mut self, face: FontFaceBytes) -> Self {
        self.fonts.push(face);
        self
    }

    /// Sets the root to a reactive Rust view `builder`, re-invoked each reactive
    /// frame — exactly the shape
    /// [`UiRuntime::set_root_view`](crate::runtime::UiRuntime::set_root_view) takes.
    /// Overrides any previous [`view`](Self::view) / [`screen`](Self::screen).
    pub fn view(mut self, builder: impl Fn() -> View + 'static) -> Self {
        self.root = Some(AppRoot::View(Box::new(builder)));
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/view_with.md")]
    pub fn view_with(mut self, builder: impl FnMut(&FrameInfo) -> View + 'static) -> Self {
        self.root = Some(AppRoot::ViewWith(Box::new(builder)));
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/virtual_list.md")]
    pub fn virtual_list(
        mut self,
        id: InternedSymbol,
        build: impl FnMut(usize) -> ViewNode + 'static,
    ) -> Self {
        self.virtual_lists.push((id, Box::new(build)));
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/on_row_click.md")]
    pub fn on_row_click(
        mut self,
        id: InternedSymbol,
        handler: impl FnMut(usize, u32) + 'static,
    ) -> Self {
        self.virtual_clicks.push((id, Box::new(handler)));
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/on_row_long_press.md")]
    pub fn on_row_long_press(
        mut self,
        id: InternedSymbol,
        handler: impl FnMut(usize) + 'static,
    ) -> Self {
        self.virtual_holds.push((id, Box::new(handler)));
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/on_scroll.md")]
    pub fn on_scroll(mut self, handler: impl FnMut(f32) + 'static) -> Self {
        self.scroll = Some(Box::new(handler));
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/painter.md")]
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

    #[doc = include_str!("../../docs/api/app/app.doc/painter_id.md")]
    pub fn painter_id(
        self,
        id: InternedSymbol,
        painter: impl FnMut(&mut CanvasRegion<'_>, &CanvasPaint<'_>) -> bool + 'static,
    ) -> Self {
        self.control
            .with_runtime(move |runtime| runtime.register_canvas(id, Box::new(painter)));
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/screen.md")]
    pub fn screen(mut self, okui_src: &str) -> Self {
        self.root = Some(AppRoot::Screen(okui_src.to_owned()));
        self
    }

    /// Adds a component contract to the runtime .okui compiler. Register its component factory separately with App::component.
    pub fn screen_contract(mut self, surface: ComponentSurface) -> Self {
        self.screen_contracts.push(surface);
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/screen_data_2.md")]
    #[must_use]
    pub fn screen_data(&self) -> ScreenData {
        self.screen_data.clone()
    }

    #[doc = include_str!("../../docs/api/app/app.doc/control_2.md")]
    #[must_use]
    pub fn control(&self) -> AppControl {
        self.control.clone()
    }

    #[doc = include_str!("../../docs/api/app/app.doc/diagnostics.md")]
    pub fn diagnostics(mut self, sink: impl obscur_diagnostics::DiagnosticSink + 'static) -> Self {
        self.diagnostics = Some(Box::new(sink));
        self
    }

    /// Installs the provider's CSS variables at the system-default origin. If the provider supplies a watch, changes wake the application and refresh the variables on its event loop. Hosts can also provide snapshots through AppControl::set_system_appearance.
    pub fn system_theme(mut self, provider: impl AppearanceProvider + 'static) -> Self {
        self.appearance_provider = Some(Box::new(provider));
        self
    }

    /// Registers an application component before the first view is built. Startup returns an error if its identifier is already registered.
    pub fn component(mut self, registration: ComponentRegistration) -> Self {
        self.components.push(registration);
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/on_command.md")]
    pub fn on_command<M>(
        mut self,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) -> Self {
        self.commands
            .push((command, handler.into_command_handler()));
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/on_command_at.md")]
    pub fn on_command_at<M>(
        self,
        target: SelectorTarget,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) -> Self {
        self.control.on_command_at(target, command, handler);
        self
    }

    #[doc = include_str!("../../docs/api/app/app.doc/add_event_listener.md")]
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

    #[doc = include_str!("../../docs/api/app/app.doc/run.md")]
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
            Some(AppRoot::View(builder)) => builder,
            Some(AppRoot::Screen(src)) => {
                // Compiled against the table, so the names the application bound
                // before `run` are declared to the compiler and a reference to
                // anything else is a §2.13 diagnostic here rather than a value that
                // never arrives.
                let mut registry = crate::okui_compiler::compile::ContractRegistry::builtins();
                for surface in &self.screen_contracts {
                    registry.register_surface(surface);
                }
                let screen = OkuiScreen::compile_with_registry(&src, screen_data, registry)?;
                Box::new(move || screen.view())
            }
            // The frame-aware builder is wrapped by the windowed runner, which owns
            // the window and therefore the only authoritative viewport.
            Some(AppRoot::ViewWith(builder)) => {
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

    #[doc = include_str!("../../docs/api/app/app.doc/into_parts.md")]
    fn into_parts(self) -> AppParts {
        AppParts {
            render_backend: self.render_backend,
            background_blur: self.background_blur,
            title: self.title,
            app_id: self.app_id,
            #[cfg(feature = "window-winit")]
            decorated: self.decorated,
            #[cfg(feature = "window-winit")]
            resize: self.resize,
            #[cfg(feature = "window-winit")]
            window_mode: self.window_mode,
            #[cfg(feature = "window-winit")]
            prefer_opengl: self.prefer_opengl,
            size: self.size,
            #[cfg(feature = "window-winit")]
            size_limits: self.size_limits,
            #[cfg(feature = "window-winit")]
            window_waker: self.window_waker,
            stylesheets: self.stylesheets,
            appearance_provider: self.appearance_provider,
            fonts: self.fonts,
            commands: self.commands,
            components: self.components,
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
    use super::{App, AppRoot, ScreenData, View};
    use crate::headless::input::SelectorTarget;
    use crate::identity::InternedSymbol;
    use crate::interaction::event::{ListenerPhase, UiEvent};

    /// A command symbol the builder cases below register against.
    const PICK: InternedSymbol = InternedSymbol::from_raw(0x5A01);

    #[cfg(feature = "window-winit")]
    #[test]
    fn size_limits_reach_the_window_configuration() {
        let parts = App::new("Window")
            .with_min_size(400.0, 300.0)
            .with_max_size(900.0, 700.0)
            .into_parts();
        assert_eq!(parts.size_limits.minimum, Some((400.0, 300.0)));
        assert_eq!(parts.size_limits.maximum, Some((900.0, 700.0)));
    }

    #[cfg(feature = "window-winit")]
    #[test]
    fn resize_defaults_to_enabled_and_can_be_disabled() {
        assert!(App::new("Default").into_parts().resize);
        assert!(!App::new("Fixed").with_resize(false).into_parts().resize);
    }

    #[cfg(feature = "window-winit")]
    #[test]
    fn saved_window_placement_reaches_startup_configuration() {
        let placement =
            serde_json::from_str(r#"{"version":1,"normal_size":[780,580],"mode":"fullscreen"}"#)
                .unwrap();
        let parts = App::new("Restore")
            .with_window_placement(placement)
            .into_parts();
        assert_eq!(parts.size, (780.0, 580.0));
        assert_eq!(parts.window_mode, crate::WindowMode::Fullscreen);

        let unknown =
            serde_json::from_str(r#"{"version":2,"normal_size":[780,580],"mode":"fullscreen"}"#)
                .unwrap();
        let parts = App::new("Restore")
            .with_size(500.0, 400.0)
            .with_window_placement(unknown)
            .into_parts();
        assert_eq!(parts.size, (500.0, 400.0));
        assert_eq!(parts.window_mode, crate::WindowMode::Normal);
    }

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

        let booted = parts
            .boot(Box::new(View::default), super::Size::new(100.0, 100.0))
            .unwrap();
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
    fn app_appearance_provider_and_control_update_style() {
        use crate::capability::SupportLevel;
        use crate::identity::ComponentId;
        use crate::system::appearance::{
            AppearanceProvider, ColorScheme, Contrast, SystemAppearance,
        };
        use crate::view::view::ViewNode;

        struct Provider;
        impl AppearanceProvider for Provider {
            fn appearance(&self) -> SystemAppearance {
                SystemAppearance {
                    window_radius: Some(12.0),
                    color_scheme: Some(ColorScheme::Dark),
                    contrast: Some(Contrast::High),
                    reduced_motion: Some(true),
                    ..SystemAppearance::default()
                }
            }
            fn support(&self) -> SupportLevel {
                SupportLevel::Native
            }
        }

        let mut app = App::new("system theme")
            .system_theme(Provider)
            .stylesheet(
                ":root { width: var(--obscur-sys-window-radius, 3px); height: 10px; }                  @media (prefers-color-scheme: dark) and (prefers-reduced-motion: reduce)                     and (prefers-contrast: more) { :root { height: 17px; } }",
            )
            .into_parts()
            .boot(
                Box::new(|| View::single(ViewNode::element(ComponentId::from_raw(1)))),
                super::Size::new(100.0, 100.0),
            )
            .unwrap();
        app.runtime.run_due_frame_reactive().expect("initial frame");
        let root = app.runtime.tree().root().unwrap();
        let node = app.runtime.tree().get(root).unwrap().child_nodes()[0];
        assert_eq!(
            app.runtime.layout_boxes()[&node].size(),
            super::Size::new(12.0, 17.0)
        );

        app.control.set_system_appearance(SystemAppearance {
            window_radius: Some(24.0),
            ..SystemAppearance::default()
        });
        app.control.apply(&mut app.runtime, &mut app.css);
        app.runtime
            .run_due_frame_reactive()
            .expect("theme update schedules a frame");
        assert_eq!(
            app.runtime.layout_boxes()[&node].size(),
            super::Size::new(24.0, 10.0)
        );
    }

    #[test]
    fn app_component_reports_a_registry_conflict_at_boot() {
        use crate::bootstrap::BuiltinComponents;
        use crate::view::component::RegistryError;

        let registration = BuiltinComponents::registrations().remove(0);
        let expected = registration.id();
        let result = App::new("component registration")
            .component(registration)
            .into_parts()
            .boot(Box::new(View::default), super::Size::new(100.0, 100.0));
        assert!(matches!(
            result,
            Err(super::AppError::ComponentRegistration(RegistryError::DuplicateComponent(id)))
                if id == expected
        ));
    }

    #[test]
    fn screen_then_view_lets_the_last_root_win() {
        // Both set the root; the last call wins, so a later `.view` clears a screen.
        let app = App::new("t").screen("<div/>").view(View::default);
        assert!(matches!(app.root, Some(AppRoot::View(_))));
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
