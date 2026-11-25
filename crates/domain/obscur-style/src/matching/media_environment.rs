//! The live environment `@media`/`@container` conditions are evaluated against
//! at match time: [`MediaEnvironment`].

use super::{MediaInput, MediaPreferences};

#[doc = include_str!("../../docs/api/matching/media_environment.doc/MediaEnvironment.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MediaEnvironment<'a> {
    containers: Option<&'a super::ContainerQueryEnvironment>,
    subject: Option<obscur_identity::NodeId>,
    selected_container: Option<obscur_identity::NodeId>,
    viewport_width: Option<f32>,
    viewport_height: Option<f32>,
    container_dimensions: [Option<f32>; 4],
    preferences: MediaPreferences,
    resolution_dppx: Option<f32>,
    input: Option<MediaInput>,
}

impl<'a> MediaEnvironment<'a> {
    /// The environment with every axis unknown; every condition holds.
    #[must_use]
    pub const fn unbounded() -> Self {
        Self {
            containers: None,
            subject: None,
            selected_container: None,
            viewport_width: None,
            viewport_height: None,
            container_dimensions: [None; 4],
            resolution_dppx: None,
            input: None,
            preferences: MediaPreferences {
                dark: false,
                reduced_motion: false,
                increased_contrast: false,
            },
        }
    }

    /// An environment with a known viewport `width`/`height` (logical px) and no
    /// container size.
    #[must_use]
    pub const fn viewport(width: f32, height: f32) -> Self {
        Self {
            containers: None,
            subject: None,
            selected_container: None,
            viewport_width: Some(width),
            viewport_height: Some(height),
            container_dimensions: [None; 4],
            resolution_dppx: Some(1.0),
            input: None,
            preferences: MediaPreferences {
                dark: false,
                reduced_motion: false,
                increased_contrast: false,
            },
        }
    }

    /// Supplies the host's pointing capabilities.
    #[must_use]
    pub const fn with_input(mut self, input: Option<MediaInput>) -> Self {
        self.input = input;
        self
    }

    pub(super) const fn input(&self) -> Option<MediaInput> {
        self.input
    }

    /// Sets the hardware pixel ratio used by screen resolution queries.
    #[must_use]
    pub const fn with_resolution_dppx(mut self, dppx: f32) -> Self {
        self.resolution_dppx = Some(dppx);
        self
    }

    pub(super) const fn resolution_dppx(&self) -> Option<f32> {
        self.resolution_dppx
    }

    /// Uses the supplied screen media preferences when evaluating rules.
    #[must_use]
    pub const fn with_preferences(mut self, preferences: MediaPreferences) -> Self {
        self.preferences = preferences;
        self
    }

    pub(super) const fn preferences(&self) -> MediaPreferences {
        self.preferences
    }

    /// Returns a copy with the querying container's `width`/`height` (logical
    /// px) set, so `@container` size queries can be evaluated.
    #[must_use]
    pub const fn with_container(mut self, width: f32, height: f32) -> Self {
        self.container_dimensions = [Some(width), Some(height), Some(width), Some(height)];
        self
    }

    /// Associates a measured snapshot with this matching environment.
    #[must_use]
    pub const fn with_query_containers(
        mut self,
        snapshot: &'a super::ContainerQueryEnvironment,
    ) -> Self {
        self.containers = Some(snapshot);
        self
    }

    /// Selects the element whose container ancestors are queried.
    #[must_use]
    pub const fn for_subject(mut self, node: obscur_identity::NodeId) -> Self {
        self.subject = Some(node);
        self
    }

    /// Returns per-axis container unit bases for the subject, in width, height, inline, block order.
    #[must_use]
    pub fn query_unit_dimensions(&self, node: obscur_identity::NodeId) -> [Option<f32>; 4] {
        self.containers
            .map_or(self.container_dimensions, |snapshot| {
                snapshot.unit_dimensions(node)
            })
    }

    pub(super) fn resolve_container(
        &self,
        name: Option<&str>,
        required: [bool; 4],
        style: bool,
    ) -> Option<Self> {
        if let Some(snapshot) = self.containers {
            let subject = self.subject?;
            let (node, dimensions) = if style && !required.into_iter().any(|axis| axis) {
                (snapshot.find_style(subject, name)?, [None; 4])
            } else {
                let (node, metrics) = snapshot.find(subject, name, required)?;
                (node, metrics.dimensions())
            };
            return Some(Self {
                selected_container: Some(node),
                container_dimensions: dimensions,
                ..*self
            });
        }
        if style {
            return None;
        }
        (name.is_none()
            && super::container_query_environment::supports_dimensions(
                self.container_dimensions,
                required,
            ))
        .then_some(*self)
    }

    pub(super) fn matches_container_style(
        &self,
        property: obscur_identity::InternedSymbol,
        expected: Option<&crate::property::PropertyValue>,
    ) -> bool {
        self.containers
            .zip(self.selected_container)
            .is_some_and(|(snapshot, node)| snapshot.matches_style(node, property, expected))
    }

    pub(super) fn container_style_range(
        &self,
        property: obscur_identity::InternedSymbol,
    ) -> Option<&super::custom_range_value::CustomRangeValue> {
        self.containers?
            .style_range(self.selected_container?, property)
    }

    pub(super) fn resolve_style_range_length(
        &self,
        length: crate::property::Length,
    ) -> Option<f32> {
        self.containers?
            .resolve_style_range_length(self.selected_container?, length)
    }

    pub(super) fn resolve_style_range_math(
        &self,
        expr: &crate::property::LengthExpr,
    ) -> Option<super::StyleRangeValue> {
        self.containers?
            .resolve_style_range_math(self.selected_container?, expr)
    }

    pub(super) fn test_container_property(
        &self,
        property: crate::property::PropertyId,
        expected: Option<&super::StylePropertyValue>,
    ) -> Option<bool> {
        self.containers?
            .test_property(self.selected_container?, property, expected)
    }

    pub(super) const fn container_size(&self, index: usize) -> Option<f32> {
        self.container_dimensions[index]
    }

    /// The known viewport width in logical px, if any.
    #[must_use]
    pub const fn viewport_width(&self) -> Option<f32> {
        self.viewport_width
    }

    /// The known viewport height in logical px, if any.
    #[must_use]
    pub const fn viewport_height(&self) -> Option<f32> {
        self.viewport_height
    }

    /// The known container width in logical px, if any.
    #[must_use]
    pub const fn container_width(&self) -> Option<f32> {
        self.container_size(0)
    }

    /// The known container height in logical px, if any.
    #[must_use]
    pub const fn container_height(&self) -> Option<f32> {
        self.container_size(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbounded_has_no_known_axes() {
        let env = MediaEnvironment::unbounded();
        assert_eq!(env.viewport_width(), None);
        assert_eq!(env.viewport_height(), None);
        assert_eq!(env.container_width(), None);
        assert_eq!(env.container_height(), None);
        assert_eq!(env, MediaEnvironment::default());
    }

    #[test]
    fn viewport_sets_width_and_height_only() {
        let env = MediaEnvironment::viewport(1000.0, 800.0);
        assert_eq!(env.viewport_width(), Some(1000.0));
        assert_eq!(env.viewport_height(), Some(800.0));
        assert_eq!(env.container_width(), None);
    }

    #[test]
    fn with_container_sets_container_axes() {
        let env = MediaEnvironment::viewport(1000.0, 800.0).with_container(200.0, 150.0);
        assert_eq!(env.container_width(), Some(200.0));
        assert_eq!(env.container_height(), Some(150.0));
        // The viewport axes are preserved.
        assert_eq!(env.viewport_width(), Some(1000.0));
    }
}
