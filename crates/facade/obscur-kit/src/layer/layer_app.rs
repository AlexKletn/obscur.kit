//! [`LayerApp`]: the builder for a layer-surface application.

use core::fmt;

use crate::identity::InternedSymbol;
use crate::shell_layer::error::LayerShellError;
use crate::view::view::View;
use crate::windowing::layer::{
    LayerAnchor, LayerExclusiveZone, LayerKeyboardInteractivity, LayerKind, LayerOutput,
    LayerSurfaceConfig, Margins,
};

/// A desktop surface — panel, dock, bar, wallpaper or overlay — described the
/// same way an [`App`](crate::App) is, and drawn by the same toolkit.
///
/// The difference from `App` is entirely in placement: a layer surface has no
/// title, no decorations and no place in the window stack. It is anchored to
/// screen edges, sits in one of four depth bands, and can reserve space that
/// maximised windows keep clear of.
pub struct LayerApp {
    config: LayerSurfaceConfig,
    stylesheets: Vec<String>,
    root: Option<Box<dyn FnMut() -> View + 'static>>,
    commands: Vec<(InternedSymbol, Box<dyn FnMut() + 'static>)>,
}

/// Why a layer-surface application could not run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerAppError {
    /// The compositor does not implement `wlr-layer-shell`.
    ///
    /// Expected on GNOME and on any non-wlroots compositor that has not adopted
    /// the protocol. A host that also has an ordinary window path should fall
    /// back to it rather than treat this as an error.
    Unsupported,
    /// The Wayland connection failed, or a required global is missing.
    Shell(LayerShellError),
    /// No usable GPU adapter, device or surface.
    Gpu(String),
    /// [`LayerApp::view`] was never called, so there is nothing to draw.
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
    /// A surface described by an explicit [`LayerSurfaceConfig`], for anything
    /// the named constructors do not cover.
    #[must_use]
    pub fn new(config: LayerSurfaceConfig) -> Self {
        Self {
            config,
            stylesheets: Vec::new(),
            root: None,
            commands: Vec::new(),
        }
    }

    /// A bar across the top of the output, `height` logical pixels tall,
    /// reserving exactly that much space from every other surface.
    #[must_use]
    pub fn top_bar(namespace: impl Into<String>, height: u32) -> Self {
        Self::new(LayerSurfaceConfig::top_bar(namespace, height))
    }

    /// A bar across the bottom of the output, reserving its own height.
    #[must_use]
    pub fn bottom_bar(namespace: impl Into<String>, height: u32) -> Self {
        Self::new(LayerSurfaceConfig::bottom_bar(namespace, height))
    }

    /// A full-output overlay above everything, reserving nothing and ignoring
    /// every other reservation.
    #[must_use]
    pub fn overlay(namespace: impl Into<String>) -> Self {
        Self::new(LayerSurfaceConfig::overlay(namespace))
    }

    /// Sets the stacking band.
    #[must_use]
    pub fn layer(mut self, layer: LayerKind) -> Self {
        self.config.layer = layer;
        self
    }

    /// Sets the anchored edges. Anchoring to both edges of an axis stretches the
    /// surface along it and overrides the requested size on that axis.
    #[must_use]
    pub fn anchor(mut self, anchor: LayerAnchor) -> Self {
        self.config.anchor = anchor;
        self
    }

    /// Sets how much space the surface reserves from others.
    #[must_use]
    pub fn exclusive_zone(mut self, zone: LayerExclusiveZone) -> Self {
        self.config.exclusive_zone = zone;
        self
    }

    /// Sets the gaps between the surface and the edges it is anchored to.
    #[must_use]
    pub fn margin(mut self, margin: Margins) -> Self {
        self.config.margin = margin;
        self
    }

    /// Places the surface on a particular output, by connector name
    /// (`"eDP-1"`, `"DP-2"`, …). The default is the compositor's choice.
    ///
    /// A name matching nothing connected falls back to that default rather than
    /// failing: unplugging a monitor must not stop a panel from appearing.
    #[must_use]
    pub fn output(mut self, output: LayerOutput) -> Self {
        self.config.output = output;
        self
    }

    /// Sets whether the surface takes keyboard focus. The default is never.
    #[must_use]
    pub fn keyboard(mut self, interactivity: LayerKeyboardInteractivity) -> Self {
        self.config.keyboard_interactivity = interactivity;
        self
    }

    /// Appends a stylesheet. Callable repeatedly; later sheets cascade over
    /// earlier ones.
    #[must_use]
    pub fn stylesheet(mut self, css: &str) -> Self {
        self.stylesheets.push(css.to_owned());
        self
    }

    /// Sets the root view builder, re-invoked each frame.
    #[must_use]
    pub fn view(mut self, builder: impl FnMut() -> View + 'static) -> Self {
        self.root = Some(Box::new(builder));
        self
    }

    /// Registers the handler for a command bound in the view or the sheet.
    #[must_use]
    pub fn on_command(mut self, command: InternedSymbol, handler: impl FnMut() + 'static) -> Self {
        self.commands.push((command, Box::new(handler)));
        self
    }

    /// Opens the surface and runs until the compositor closes it.
    ///
    /// # Errors
    ///
    /// [`LayerAppError::Unsupported`] where the compositor has no layer shell,
    /// [`LayerAppError::Shell`] for a connection failure,
    /// [`LayerAppError::Gpu`] where no adapter can be acquired, and
    /// [`LayerAppError::NoRoot`] when no root view was set.
    pub fn run(self) -> Result<(), LayerAppError> {
        let root = self.root.ok_or(LayerAppError::NoRoot)?;
        super::runner::run(self.config, self.stylesheets, root, self.commands)
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
