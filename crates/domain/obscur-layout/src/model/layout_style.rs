//! Layout's own per-node input style.

use crate::algorithm::{GridAutoFlow, GridPlacement};
use crate::model::{
    AlignItems, AlignSelf, Dimension, FlexDirection, Inset, JustifyContent, LayoutDisplay, Position,
};
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

    /// The node's positioning scheme (`position`).
    ///
    /// [`Position::Static`] (the default) lays the node out in normal flow and
    /// ignores [`inset`](Self::inset). [`Position::Relative`] keeps the node in
    /// flow but offsets its final position (and its whole subtree) by
    /// [`inset`](Self::inset). [`Position::Absolute`] / [`Position::Fixed`] are
    /// placeholders treated as [`Position::Static`] until out-of-flow layout
    /// lands.
    pub position: Position,

    /// The node's positioning insets (`top` / `right` / `bottom` / `left`).
    ///
    /// Only consulted for a [`Position::Relative`] (later, out-of-flow) node;
    /// the default is all-`auto` ([`Inset::AUTO`]), which contributes no offset.
    pub inset: Inset,

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
    /// Flex container: main-axis distribution of items and free space. Also the
    /// grid container's *inline-axis* (column) distribution of leftover track
    /// space (`justify-content`): when the columns' total size is smaller than
    /// the content width the free space is distributed per this value.
    pub justify_content: JustifyContent,
    /// Grid container: *block-axis* (row) distribution of leftover track space
    /// (`align-content`): when the rows' total size is smaller than the content
    /// height the free space is distributed per this value. Ignored by flex.
    /// Defaults to [`JustifyContent::FlexStart`] (no distribution), so a default
    /// grid packs its rows at the top exactly as before.
    pub align_content: JustifyContent,
    /// Flex container: cross-axis alignment of items. Also the grid container's
    /// default *block-axis* (`align-items`) alignment of each item within its
    /// cell.
    pub align_items: AlignItems,
    /// Grid container: default *inline-axis* (`justify-items`) alignment of each
    /// item within its cell. Ignored by flex. Default [`AlignItems::Stretch`].
    pub justify_items: AlignItems,
    /// Grid item: per-item *block-axis* (`align-self`) alignment override.
    /// [`AlignSelf::Auto`] (the default) inherits the container's `align-items`.
    pub align_self: AlignSelf,
    /// Grid item: per-item *inline-axis* (`justify-self`) alignment override.
    /// [`AlignSelf::Auto`] (the default) inherits the container's `justify-items`.
    pub justify_self: AlignSelf,
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

    /// Grid container: the auto-placement flow mode (`grid-auto-flow`).
    ///
    /// Only meaningful for a grid container. The default is
    /// [`GridAutoFlow::Row`] (row-major, sparse) — the grid's historical
    /// behaviour — so a default style auto-places exactly as before.
    pub grid_auto_flow: GridAutoFlow,
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
            position: Position::Static,
            inset: Inset::AUTO,
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
            align_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            justify_items: AlignItems::Stretch,
            align_self: AlignSelf::Auto,
            justify_self: AlignSelf::Auto,
            gap: 0.0,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Dimension::Auto,
            font_size: Self::DEFAULT_FONT_SIZE,
            grid_placement: GridPlacement::default(),
            grid_auto_flow: GridAutoFlow::default(),
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

    /// Returns a copy with the positioning scheme `position` replaced.
    #[must_use]
    pub const fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Returns a copy with the positioning `inset` replaced.
    #[must_use]
    pub const fn with_inset(mut self, inset: Inset) -> Self {
        self.inset = inset;
        self
    }

    /// The offset a [`Position::Relative`] node adds to its final in-flow
    /// position (and to its whole subtree), in logical pixels (spec §4.1).
    ///
    /// Returns `(dx, dy)`, computed from the node's [`inset`](Self::inset)
    /// against the containing block size (`containing_width` / `containing_height`,
    /// the parent content box). Per CSS `left` wins over `right` and `top` over
    /// `bottom`:
    ///
    /// * `dx` = `left` if definite, else `-right` if definite, else `0`.
    /// * `dy` = `top` if definite, else `-bottom` if definite, else `0`.
    ///
    /// A positive `left` moves the node right; a positive `right` moves it left
    /// (and likewise for `top` / `bottom` on the y axis). Any non-[`Relative`]
    /// scheme — including the [`Position::Absolute`] / [`Position::Fixed`]
    /// placeholders, which are treated as [`Position::Static`] this step —
    /// returns `(0.0, 0.0)`, so a default node is never perturbed.
    #[must_use]
    pub fn relative_offset(&self, containing_width: f32, containing_height: f32) -> (f32, f32) {
        if !self.position.is_relative() {
            return (0.0, 0.0);
        }
        let dx = self.inset.left.resolve(containing_width).or_else(|| {
            self.inset
                .right
                .resolve(containing_width)
                .map(|right| -right)
        });
        let dy = self.inset.top.resolve(containing_height).or_else(|| {
            self.inset
                .bottom
                .resolve(containing_height)
                .map(|bottom| -bottom)
        });
        (dx.unwrap_or(0.0), dy.unwrap_or(0.0))
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

    /// Returns a copy with the grid container `grid_auto_flow` replaced.
    #[must_use]
    pub const fn with_grid_auto_flow(mut self, grid_auto_flow: GridAutoFlow) -> Self {
        self.grid_auto_flow = grid_auto_flow;
        self
    }

    /// The grid container's auto-placement flow mode.
    #[must_use]
    pub const fn grid_auto_flow(&self) -> GridAutoFlow {
        self.grid_auto_flow
    }

    /// Returns a copy with the grid container `justify_items` default replaced.
    #[must_use]
    pub const fn with_justify_items(mut self, justify_items: AlignItems) -> Self {
        self.justify_items = justify_items;
        self
    }

    /// Returns a copy with the grid container `align_items` default replaced.
    #[must_use]
    pub const fn with_align_items(mut self, align_items: AlignItems) -> Self {
        self.align_items = align_items;
        self
    }

    /// Returns a copy with the grid item `align_self` override replaced.
    #[must_use]
    pub const fn with_align_self(mut self, align_self: AlignSelf) -> Self {
        self.align_self = align_self;
        self
    }

    /// Returns a copy with the grid item `justify_self` override replaced.
    #[must_use]
    pub const fn with_justify_self(mut self, justify_self: AlignSelf) -> Self {
        self.justify_self = justify_self;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutStyle;
    use crate::model::{
        AlignItems, AlignSelf, Dimension, FlexDirection, Inset, JustifyContent, LayoutDisplay,
        Position,
    };
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
        assert_eq!(style.align_content, JustifyContent::FlexStart);
        assert_eq!(style.align_items, AlignItems::Stretch);
        assert_eq!(style.justify_items, AlignItems::Stretch);
        assert_eq!(style.align_self, AlignSelf::Auto);
        assert_eq!(style.justify_self, AlignSelf::Auto);
        assert_eq!(style.gap, 0.0);
        assert_eq!(style.flex_grow, 0.0);
        assert_eq!(style.flex_shrink, 1.0);
        assert!(style.flex_basis.is_auto());
        assert_eq!(style.font_size, LayoutStyle::DEFAULT_FONT_SIZE);
        // Positioning defaults: `static` with all-`auto` insets (no offset).
        assert_eq!(style.position, Position::Static);
        assert_eq!(style.inset, Inset::AUTO);
    }

    #[test]
    fn static_node_has_no_relative_offset() {
        // The default (static) node contributes no offset, regardless of insets.
        let style = LayoutStyle::default().with_inset(Inset::new(
            Dimension::Px(10.0),
            Dimension::Auto,
            Dimension::Auto,
            Dimension::Px(5.0),
        ));
        assert_eq!(style.relative_offset(200.0, 100.0), (0.0, 0.0));
    }

    #[test]
    fn relative_offset_uses_top_and_left() {
        // `top: 10px; left: 5px` shifts the node by (+5, +10).
        let style = LayoutStyle::default()
            .with_position(Position::Relative)
            .with_inset(Inset::new(
                Dimension::Px(10.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Px(5.0),
            ));
        assert_eq!(style.relative_offset(200.0, 100.0), (5.0, 10.0));
    }

    #[test]
    fn relative_offset_right_and_bottom_move_opposite() {
        // With no `left` / `top`, a positive `right` / `bottom` moves the node the
        // other way: `right: 5px; bottom: 10px` => (-5, -10).
        let style = LayoutStyle::default()
            .with_position(Position::Relative)
            .with_inset(Inset::new(
                Dimension::Auto,
                Dimension::Px(5.0),
                Dimension::Px(10.0),
                Dimension::Auto,
            ));
        assert_eq!(style.relative_offset(200.0, 100.0), (-5.0, -10.0));
    }

    #[test]
    fn relative_offset_left_wins_over_right() {
        // Both edges definite: `left` wins over `right`, `top` over `bottom`.
        let style = LayoutStyle::default()
            .with_position(Position::Relative)
            .with_inset(Inset::new(
                Dimension::Px(3.0),
                Dimension::Px(99.0),
                Dimension::Px(99.0),
                Dimension::Px(7.0),
            ));
        assert_eq!(style.relative_offset(200.0, 100.0), (7.0, 3.0));
    }

    #[test]
    fn relative_offset_resolves_percent_against_containing_block() {
        // Percent insets resolve against the containing block size.
        let style = LayoutStyle::default()
            .with_position(Position::Relative)
            .with_inset(Inset::new(
                Dimension::Percent(10.0),
                Dimension::Auto,
                Dimension::Auto,
                Dimension::Percent(25.0),
            ));
        // left = 25% of 200 = 50; top = 10% of 100 = 10.
        assert_eq!(style.relative_offset(200.0, 100.0), (50.0, 10.0));
    }

    #[test]
    fn absolute_and_fixed_are_static_this_step() {
        // Out-of-flow placeholders contribute no offset until step 2 wires them.
        for position in [Position::Absolute, Position::Fixed] {
            let style = LayoutStyle::default()
                .with_position(position)
                .with_inset(Inset::new(
                    Dimension::Px(10.0),
                    Dimension::Auto,
                    Dimension::Auto,
                    Dimension::Px(5.0),
                ));
            assert_eq!(style.relative_offset(200.0, 100.0), (0.0, 0.0));
        }
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
