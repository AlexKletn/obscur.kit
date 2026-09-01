#![doc = include_str!("resolution_context.doc/_module.md")]

use crate::computed::initial_value::INITIAL_FONT_SIZE;
use crate::computed::{ChMetrics, CustomValueParser};
use crate::property::EnvironmentValues;

#[doc = include_str!("resolution_context.doc/ResolutionContext.md")]
#[derive(Clone, Copy)]
pub struct ResolutionContext<'a> {
    #[doc = include_str!("resolution_context.doc/root_font_size.md")]
    pub root_font_size: f32,
    #[doc = include_str!("resolution_context.doc/viewport_width.md")]
    pub viewport_width: f32,
    #[doc = include_str!("resolution_context.doc/viewport_height.md")]
    pub viewport_height: f32,
    #[doc = include_str!("resolution_context.doc/metrics.md")]
    pub metrics: Option<&'a dyn ChMetrics>,
    #[doc = include_str!("resolution_context.doc/custom_values.md")]
    pub custom_values: Option<&'a dyn CustomValueParser>,
    #[doc = include_str!("resolution_context.doc/environment.md")]
    pub environment: EnvironmentValues,
}

impl std::fmt::Debug for ResolutionContext<'_> {
    #[doc = include_str!("resolution_context.doc/fmt.md")]
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
            .finish()
    }
}

impl<'a> ResolutionContext<'a> {
    #[doc = include_str!("resolution_context.doc/new.md")]
    #[must_use]
    pub const fn new(root_font_size: f32, viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            root_font_size,
            viewport_width,
            viewport_height,
            metrics: None,
            custom_values: None,
            environment: EnvironmentValues::none(),
        }
    }

    #[doc = include_str!("resolution_context.doc/for_root.md")]
    #[must_use]
    pub const fn for_root(viewport_width: f32, viewport_height: f32) -> Self {
        Self::new(INITIAL_FONT_SIZE, viewport_width, viewport_height)
    }

    #[doc = include_str!("resolution_context.doc/with_metrics.md")]
    #[must_use]
    pub const fn with_metrics(mut self, metrics: &'a dyn ChMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    #[doc = include_str!("resolution_context.doc/with_custom_values.md")]
    #[must_use]
    pub const fn with_custom_values(mut self, parser: &'a dyn CustomValueParser) -> Self {
        self.custom_values = Some(parser);
        self
    }

    #[doc = include_str!("resolution_context.doc/with_environment.md")]
    #[must_use]
    pub const fn with_environment(mut self, environment: EnvironmentValues) -> Self {
        self.environment = environment;
        self
    }
}
