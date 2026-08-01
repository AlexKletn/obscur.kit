#![doc = include_str!("layer_app.doc/_module.md")]

use core::fmt;

use crate::identity::InternedSymbol;
use crate::runtime::{CommandHandler, IntoCommandHandler};
use crate::shell_layer::error::LayerShellError;
use crate::view::view::View;
use crate::windowing::layer::{
    LayerAnchor, LayerExclusiveZone, LayerKeyboardInteractivity, LayerKind, LayerOutput,
    LayerSurfaceConfig, Margins,
};

#[doc = include_str!("layer_app.doc/LayerApp.md")]
pub struct LayerApp {
    config: LayerSurfaceConfig,
    stylesheets: Vec<String>,
    root: Option<Box<dyn FnMut() -> View + 'static>>,
    commands: Vec<(InternedSymbol, CommandHandler)>,
    handle: Option<super::LayerHandle>,
    waker: Option<crate::shell_layer::surface::LayerWaker>,
}

#[doc = include_str!("layer_app.doc/LayerAppError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerAppError {
    #[doc = include_str!("layer_app.doc/Unsupported.md")]
    Unsupported,
    #[doc = include_str!("layer_app.doc/Shell.md")]
    Shell(LayerShellError),
    #[doc = include_str!("layer_app.doc/Gpu.md")]
    Gpu(String),
    #[doc = include_str!("layer_app.doc/NoRoot.md")]
    NoRoot,
}

impl fmt::Display for LayerAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported => write!(
                f,
                "the compositor does not implement wlr-layer-shell (expected on GNOME)"
            ),
            Self::Shell(error) => write!(f, "{error}"),
            Self::Gpu(why) => write!(f, "gpu unavailable: {why}"),
            Self::NoRoot => write!(f, "no root view: call LayerApp::view"),
        }
    }
}

impl std::error::Error for LayerAppError {}

impl From<LayerShellError> for LayerAppError {
    fn from(error: LayerShellError) -> Self {
        match error {
            LayerShellError::Unsupported => Self::Unsupported,
            other => Self::Shell(other),
        }
    }
}

impl LayerApp {
    #[doc = include_str!("layer_app.doc/new.md")]
    #[must_use]
    pub fn new(config: LayerSurfaceConfig) -> Self {
        Self {
            config,
            stylesheets: Vec::new(),
            root: None,
            commands: Vec::new(),
            handle: None,
            waker: None,
        }
    }

    #[doc = include_str!("layer_app.doc/top_bar.md")]
    #[must_use]
    pub fn top_bar(namespace: impl Into<String>, height: u32) -> Self {
        Self::new(LayerSurfaceConfig::top_bar(namespace, height))
    }

    #[doc = include_str!("layer_app.doc/bottom_bar.md")]
    #[must_use]
    pub fn bottom_bar(namespace: impl Into<String>, height: u32) -> Self {
        Self::new(LayerSurfaceConfig::bottom_bar(namespace, height))
    }

    #[doc = include_str!("layer_app.doc/overlay.md")]
    #[must_use]
    pub fn overlay(namespace: impl Into<String>) -> Self {
        Self::new(LayerSurfaceConfig::overlay(namespace))
    }

    #[doc = include_str!("layer_app.doc/layer.md")]
    #[must_use]
    pub fn layer(mut self, layer: LayerKind) -> Self {
        self.config.layer = layer;
        self
    }

    #[doc = include_str!("layer_app.doc/anchor.md")]
    #[must_use]
    pub fn anchor(mut self, anchor: LayerAnchor) -> Self {
        self.config.anchor = anchor;
        self
    }

    #[doc = include_str!("layer_app.doc/exclusive_zone.md")]
    #[must_use]
    pub fn exclusive_zone(mut self, zone: LayerExclusiveZone) -> Self {
        self.config.exclusive_zone = zone;
        self
    }

    #[doc = include_str!("layer_app.doc/margin.md")]
    #[must_use]
    pub fn margin(mut self, margin: Margins) -> Self {
        self.config.margin = margin;
        self
    }

    #[doc = include_str!("layer_app.doc/output.md")]
    #[must_use]
    pub fn output(mut self, output: LayerOutput) -> Self {
        self.config.output = output;
        self
    }

    #[doc = include_str!("layer_app.doc/keyboard.md")]
    #[must_use]
    pub fn keyboard(mut self, interactivity: LayerKeyboardInteractivity) -> Self {
        self.config.keyboard_interactivity = interactivity;
        self
    }

    #[doc = include_str!("layer_app.doc/stylesheet.md")]
    #[must_use]
    pub fn stylesheet(mut self, css: &str) -> Self {
        self.stylesheets.push(css.to_owned());
        self
    }

    #[doc = include_str!("layer_app.doc/handle.md")]
    #[must_use]
    pub fn handle(mut self, handle: super::LayerHandle) -> Self {
        self.handle = Some(handle);
        self
    }

    #[doc = include_str!("layer_app.doc/waker.md")]
    #[must_use]
    pub fn waker(mut self, waker: crate::shell_layer::surface::LayerWaker) -> Self {
        self.waker = Some(waker);
        self
    }

    #[doc = include_str!("layer_app.doc/view.md")]
    #[must_use]
    pub fn view(mut self, builder: impl FnMut() -> View + 'static) -> Self {
        self.root = Some(Box::new(builder));
        self
    }

    #[doc = include_str!("layer_app.doc/on_command.md")]
    #[must_use]
    pub fn on_command<M>(
        mut self,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) -> Self {
        self.commands
            .push((command, handler.into_command_handler()));
        self
    }

    #[doc = include_str!("layer_app.doc/run.md")]
    pub fn run(self) -> Result<(), LayerAppError> {
        let root = self.root.ok_or(LayerAppError::NoRoot)?;
        super::runner::run(
            self.config,
            self.stylesheets,
            root,
            self.commands,
            self.handle.unwrap_or_default(),
            self.waker,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The named constructors are the reason a panel works without reading the
    /// protocol: a bar that does not reserve its height is a strip that windows
    /// slide underneath, and forgetting the reservation is the classic mistake.
    #[test]
    fn a_top_bar_reserves_its_own_height() {
        let app = LayerApp::top_bar("panel", 40);
        assert_eq!(app.config.exclusive_zone, LayerExclusiveZone::Reserve(40));
        assert_eq!(app.config.layer, LayerKind::Top);
        assert!(app.config.anchor.stretches_horizontally());
    }

    #[test]
    fn an_overlay_sits_above_everything_and_reserves_nothing() {
        let app = LayerApp::overlay("lock");
        assert_eq!(app.config.layer, LayerKind::Overlay);
        assert_eq!(app.config.exclusive_zone, LayerExclusiveZone::Ignore);
    }

    #[test]
    fn an_output_can_be_pinned_by_connector_name() {
        let app = LayerApp::top_bar("panel", 24).output(LayerOutput::Named("DP-2".to_owned()));
        assert_eq!(app.config.output.name(), Some("DP-2"));
    }

    #[test]
    fn the_builders_reach_the_config() {
        let app = LayerApp::new(LayerSurfaceConfig::new("x"))
            .layer(LayerKind::Background)
            .anchor(LayerAnchor::ALL)
            .margin(Margins::uniform(4))
            .keyboard(LayerKeyboardInteractivity::OnDemand);
        assert_eq!(app.config.layer, LayerKind::Background);
        assert!(app.config.anchor.stretches_vertically());
        assert_eq!(app.config.margin.top, 4);
        assert_eq!(
            app.config.keyboard_interactivity,
            LayerKeyboardInteractivity::OnDemand
        );
    }

    /// Running with no root is refused rather than opening an empty surface: an
    /// unexplained blank panel is worse than an error that names the missing call.
    #[test]
    fn running_without_a_root_view_is_refused_by_name() {
        let error = LayerApp::top_bar("panel", 24).run().unwrap_err();
        assert_eq!(error, LayerAppError::NoRoot);
        assert!(error.to_string().contains("LayerApp::view"));
    }

    #[test]
    fn an_unsupported_compositor_is_reported_as_a_capability() {
        let error: LayerAppError = LayerShellError::Unsupported.into();
        assert_eq!(error, LayerAppError::Unsupported);
        assert!(error.to_string().contains("GNOME"));
    }
}
