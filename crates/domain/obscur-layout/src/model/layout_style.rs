//! Layout's own per-node input style.

use crate::algorithm::GridPlacement;
use crate::model::{AlignItems, Dimension, FlexDirection, JustifyContent, LayoutDisplay};
use obscur_geometry::Insets;

/// The computed layout input for a single node (spec §2.3, §4.1).
///
/// This is layout's own vocabulary, mapped from the style crate's computed style
/// by the runtime; the layout crate does not depend on `obscur-style`. All
/// lengths are fractional logical pixels and are preserved without rounding
/// (spec §4.6).
///
/// Grid, positioning, docking, and other formatting contexts are out of scope
/// for Phase 1 and are not represented here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutStyle {
    /// The formatting context this node establishes.
    pub display: LayoutDisplay,

    /// Preferred width.
    pub width: Dimension,
    /// Preferred height.
    pub height: Dimension,
    /// Lower bound on the resolved width.
    pub min_width: Dimension,
    /// Lower bound on the resolved height.
    pub min_height: Dimension,
    /// Upper bound on the resolved width.
    pub max_width: Dimension,
    /// Upper bound on the resolved height.
    pub max_height: Dimension,

    /// Space reserved outside the border box (margin edges).
    pub margin: Insets,
    /// Space reserved between the border box and the content box (padding edges).
    pub padding: Insets,
    /// Border thickness on each edge.
    pub border: Insets,

    /// Flex container: the main-axis direction.
    pub flex_direction: FlexDirection,
    /// Flex container: main-axis distribution of items and free space.
    pub justify_content: JustifyContent,
    /// Flex container: cross-axis alignment of items.
    pub align_items: AlignItems,
    /// Flex container: gap between adjacent items, in logical pixels.
    pub gap: f32,

    /// Flex item: growth factor for distributing positive free space.
    pub flex_grow: f32,
    /// Flex item: shrink factor for absorbing negative free space.
    pub flex_shrink: f32,
    /// Flex item: base size along the main axis before grow/shrink.
    pub flex_basis: Dimension,

    /// The node's font size in logical pixels, used to shape and wrap a text
    /// leaf during the measure/arrange pass (spec §5.10).
    ///
    /// Only meaningful for a text node: the coordinator populates it from the
    /// node's computed `font-size` so the constraint-aware measure can shape and
    /// wrap the run to its available width. It defaults to
    /// [`DEFAULT_FONT_SIZE`](Self::DEFAULT_FONT_SIZE) and is ignored by every
    /// box (non-text) algorithm.
    pub font_size: f32,

    /// Grid item: explicit line-based placement (`grid-column` / `grid-row`).
    ///
    /// Only meaningful for a child of a grid container. The default is all
    /// [`GridLine::Auto`](crate::algorithm::GridLine), i.e. pure row-major
    /// auto-placement — the grid's historical behaviour — so a default style
    /// never perturbs a non-grid layout.
    pub grid_placement: GridPlacement,
}

impl LayoutStyle {
    /// The initial `font-size` (the CSS default), used for a text leaf whose
    /// [`font_size`](Self::font_size) is not otherwise set.
    pub const DEFAULT_FONT_SIZE: f32 = 16.0;
}

impl Default for LayoutStyle {
    /// CSS-like defaults: `Block` display, all sizes `Auto`, zero insets and gap,
    /// `Row` direction, `FlexStart` justification, `Stretch` alignment,
    /// `flex_grow: 0.0`, `flex_shrink: 1.0`, `Auto` flex basis, and the initial
    /// `16px` font size.
    fn default() -> Self {
        Self {
            display: LayoutDisplay::Block,
            width: Dimension::Auto,
            height: Dimension::Auto,
            min_width: Dimension::Auto,
            min_height: Dimension::Auto,
            max_width: Dimension::Auto,
            max_height: Dimension::Auto,
            margin: Insets::uniform(0.0),
            padding: Insets::uniform(0.0),
            border: Insets::uniform(0.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            gap: 0.0,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Dimension::Auto,
            font_size: Self::DEFAULT_FONT_SIZE,
            grid_placement: GridPlacement::default(),
        }
    }
}

impl LayoutStyle {
    /// Returns a copy with `display` replaced.
    #[must_use]
    pub const fn with_display(mut self, display: LayoutDisplay) -> Self {
        self.display = display;
        self
    }

    /// Returns a copy with `width` replaced.
    #[must_use]
    pub const fn with_width(mut self, width: Dimension) -> Self {
        self.width = width;
        self
    }

    /// Returns a copy with `height` replaced.
    #[must_use]
    pub const fn with_height(mut self, height: Dimension) -> Self {
        self.height = height;
        self
    }

    /// Returns a copy with `font_size` replaced.
    #[must_use]
    pub const fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Returns a copy with the grid item `grid_placement` replaced.
    #[must_use]
    pub const fn with_grid_placement(mut self, grid_placement: GridPlacement) -> Self {
        self.grid_placement = grid_placement;
        self
    }

    /// The grid item's explicit line-based placement.
    #[must_use]
    pub const fn grid_placement(&self) -> GridPlacement {
        self.grid_placement
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutStyle;
    use crate::model::{AlignItems, Dimension, FlexDirection, JustifyContent, LayoutDisplay};
    use obscur_geometry::Insets;

    #[test]
    fn default_matches_css_like_values() {
        let style = LayoutStyle::default();
        assert_eq!(style.display, LayoutDisplay::Block);
        assert!(style.width.is_auto());
        assert!(style.height.is_auto());
        assert!(style.min_width.is_auto());
        assert!(style.max_height.is_auto());
        assert_eq!(style.margin, Insets::uniform(0.0));
        assert_eq!(style.padding, Insets::uniform(0.0));
        assert_eq!(style.border, Insets::uniform(0.0));
        assert_eq!(style.flex_direction, FlexDirection::Row);
        assert_eq!(style.justify_content, JustifyContent::FlexStart);
        assert_eq!(style.align_items, AlignItems::Stretch);
        assert_eq!(style.gap, 0.0);
        assert_eq!(style.flex_grow, 0.0);
        assert_eq!(style.flex_shrink, 1.0);
        assert!(style.flex_basis.is_auto());
        assert_eq!(style.font_size, LayoutStyle::DEFAULT_FONT_SIZE);
    }

    #[test]
    fn builders_override_fields() {
        let style = LayoutStyle::default()
            .with_display(LayoutDisplay::Flex)
            .with_width(Dimension::Px(120.0))
            .with_height(Dimension::Percent(50.0));
        assert_eq!(style.display, LayoutDisplay::Flex);
        assert_eq!(style.width, Dimension::Px(120.0));
        assert_eq!(style.height, Dimension::Percent(50.0));
    }
}
