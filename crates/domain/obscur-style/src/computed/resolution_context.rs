#![doc = include_str!("../../docs/api/computed/resolution_context.doc/_module.md")]

use crate::computed::initial_value::INITIAL_FONT_SIZE;
use crate::computed::{ChMetrics, CustomValueParser};
use crate::property::{EnvironmentValues, LengthContext};

#[doc = include_str!("../../docs/api/computed/resolution_context.doc/ResolutionContext.md")]
#[derive(Clone, Copy)]
pub struct ResolutionContext<'a> {
    /// The document root's computed `font-size` in logical pixels (base for
    /// `rem`).
    pub root_font_size: f32,
    /// The viewport width in logical pixels (base for `vw`).
    pub viewport_width: f32,
    /// The viewport height in logical pixels (base for `vh`).
    pub viewport_height: f32,
    /// The port that measures the `0` glyph (base for `ch`), when the caller has
    /// one. [`None`] leaves `ch` to the specified fallbacks — see
    /// [`with_metrics`](Self::with_metrics).
    pub metrics: Option<&'a dyn ChMetrics>,
    /// The port that parses a custom property's declared text for the property it
    /// is substituted into, when the caller has one. [`None`] leaves every raw
    /// token sequence invalid at computed-value time — see
    /// [`with_custom_values`](Self::with_custom_values).
    pub custom_values: Option<&'a dyn CustomValueParser>,
    /// The host-reported environment values `env()` resolves against (spec
    /// §6.15). All zeroes unless a host reports insets.
    pub environment: EnvironmentValues,
    query_unit_dimensions: [Option<f32>; 4],
}

impl std::fmt::Debug for ResolutionContext<'_> {
    /// Reports whether a [`ChMetrics`] port is attached, not what it is: a port
    /// is opaque, and "is one attached?" is the question a `ch` bug asks.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolutionContext")
            .field("root_font_size", &self.root_font_size)
            .field("viewport_width", &self.viewport_width)
            .field("viewport_height", &self.viewport_height)
            .field("metrics", &self.metrics.map(|_| "<ChMetrics>"))
            .field(
                "custom_values",
                &self.custom_values.map(|_| "<CustomValueParser>"),
            )
            .field("environment", &self.environment)
            .field("query_unit_dimensions", &self.query_unit_dimensions)
            .finish()
    }
}

impl<'a> ResolutionContext<'a> {
    /// Builds a context whose `rem` base is `root_font_size`, with no font
    /// metrics — see [`with_metrics`](Self::with_metrics).
    #[must_use]
    pub const fn new(root_font_size: f32, viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            root_font_size,
            viewport_width,
            viewport_height,
            metrics: None,
            custom_values: None,
            environment: EnvironmentValues::none(),
            query_unit_dimensions: [None; 4],
        }
    }

    #[doc = include_str!("../../docs/api/computed/resolution_context.doc/for_root.md")]
    #[must_use]
    pub const fn for_root(viewport_width: f32, viewport_height: f32) -> Self {
        Self::new(INITIAL_FONT_SIZE, viewport_width, viewport_height)
    }

    #[doc = include_str!("../../docs/api/computed/resolution_context.doc/with_metrics.md")]
    #[must_use]
    pub const fn with_metrics(mut self, metrics: &'a dyn ChMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    #[doc = include_str!("../../docs/api/computed/resolution_context.doc/with_custom_values.md")]
    #[must_use]
    pub const fn with_custom_values(mut self, parser: &'a dyn CustomValueParser) -> Self {
        self.custom_values = Some(parser);
        self
    }

    /// Supplies query container bases for the current node, in width, height, inline, block order. Missing axes use the viewport.
    #[must_use]
    pub const fn with_query_unit_dimensions(mut self, dimensions: [Option<f32>; 4]) -> Self {
        self.query_unit_dimensions = dimensions;
        self
    }

    pub(super) fn lengths(self, font_size: f32, vertical: bool) -> LengthContext {
        LengthContext::new(
            font_size,
            self.root_font_size,
            self.viewport_width,
            self.viewport_height,
        )
        .with_environment(self.environment)
        .with_query_unit_dimensions(self.query_unit_dimensions, vertical)
    }

    /// Attaches the host-reported environment values, so `env(safe-area-inset-*)`
    /// and the keyboard insets resolve to what the host measured rather than to
    /// the no-cutout default (spec §6.15).
    #[must_use]
    pub const fn with_environment(mut self, environment: EnvironmentValues) -> Self {
        self.environment = environment;
        self
    }
}
