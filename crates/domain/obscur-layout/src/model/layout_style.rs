//! Layout's own per-node input style.

use crate::algorithm::{GridAutoFlow, GridPlacement};
use crate::model::{
    AlignItems, AlignSelf, BoxSizing, Dimension, Direction, FlexDirection, FlexWrap, Inset,
    JustifyContent, LayoutDisplay, MarginEdges, Position, SizeContainment, WritingMode,
};
use obscur_geometry::{Insets, Size};

#[doc = include_str!("../../docs/api/model/layout_style.doc/LayoutStyle.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutStyle {
    /// The formatting context this node establishes.
    pub display: LayoutDisplay,

    #[doc = include_str!("../../docs/api/model/layout_style.doc/direction.md")]
    pub direction: Direction,

    #[doc = include_str!("../../docs/api/model/layout_style.doc/writing_mode.md")]
    pub writing_mode: WritingMode,

    /// The logical axes whose used size is determined independently of descendants.
    pub size_containment: SizeContainment,

    /// The explicit intrinsic content size in physical axes, used only on size-contained axes.
    pub contained_intrinsic_size: Size,

    #[doc = include_str!("../../docs/api/model/layout_style.doc/position.md")]
    pub position: Position,

    #[doc = include_str!("../../docs/api/model/layout_style.doc/inset.md")]
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

    #[doc = include_str!("../../docs/api/model/layout_style.doc/box_sizing.md")]
    pub box_sizing: BoxSizing,

    /// Space reserved outside the border box (margin edges). Each edge may be
    /// `auto`, which absorbs leftover space rather than reserving a fixed amount
    /// — the `margin: 0 auto` centring rule (CSS 2.1 §10.3.3).
    pub margin: MarginEdges,
    #[doc = include_str!("../../docs/api/model/layout_style.doc/padding.md")]
    pub padding: Insets,
    /// Padding lengths that use a containing inline dimension.
    pub padding_lengths: Option<MarginEdges>,
    /// Border thickness on each edge.
    pub border: Insets,

    /// Flex container: the main-axis direction.
    pub flex_direction: FlexDirection,
    /// Flex container: whether items are confined to a single line or wrap onto
    /// multiple lines along the cross axis (`flex-wrap`). Default
    /// [`FlexWrap::NoWrap`] (single-line), so a container that never sets it is
    /// byte-for-byte identical to before wrapping existed. Ignored by non-flex
    /// formatting contexts.
    pub flex_wrap: FlexWrap,
    /// Flex container: main-axis distribution of items and free space. Also the
    /// grid container's *inline-axis* (column) distribution of leftover track
    /// space (`justify-content`): when the columns' total size is smaller than
    /// the content width the free space is distributed per this value.
    pub justify_content: JustifyContent,
    /// Distributes leftover block-axis space among grid rows or cross-axis space
    /// among wrapped flex lines. The default [`JustifyContent::Normal`] stretches
    /// eligible automatic grid rows and wrapped flex lines.
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
    /// An authored gap whose percentage basis is supplied by the layout axis.
    pub gap_dimension: Option<Dimension>,
    /// The authored gap between rows, when specified.
    pub row_gap: Option<Dimension>,
    /// The authored gap between columns, when specified.
    pub column_gap: Option<Dimension>,
    /// Relative placement order among sibling layout items.
    pub order: i32,

    /// Flex item: growth factor for distributing positive free space.
    pub flex_grow: f32,
    /// Flex item: shrink factor for absorbing negative free space.
    pub flex_shrink: f32,
    /// Flex item: base size along the main axis before grow/shrink.
    pub flex_basis: Dimension,

    #[doc = include_str!("../../docs/api/model/layout_style.doc/font_size.md")]
    pub font_size: f32,

    #[doc = include_str!("../../docs/api/model/layout_style.doc/grid_placement.md")]
    pub grid_placement: GridPlacement,

    #[doc = include_str!("../../docs/api/model/layout_style.doc/grid_auto_flow.md")]
    pub grid_auto_flow: GridAutoFlow,

    #[doc = include_str!("../../docs/api/model/layout_style.doc/scroll_container.md")]
    pub scroll_container: bool,
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
            direction: Direction::Ltr,
            writing_mode: WritingMode::HorizontalTb,
            size_containment: SizeContainment::None,
            contained_intrinsic_size: Size::ZERO,
            position: Position::Static,
            inset: Inset::AUTO,
            width: Dimension::Auto,
            height: Dimension::Auto,
            min_width: Dimension::Auto,
            min_height: Dimension::Auto,
            max_width: Dimension::Auto,
            max_height: Dimension::Auto,
            box_sizing: BoxSizing::ContentBox,
            margin: MarginEdges::ZERO,
            padding: Insets::uniform(0.0),
            padding_lengths: None,
            border: Insets::uniform(0.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::Normal,
            align_content: JustifyContent::Normal,
            align_items: AlignItems::Stretch,
            justify_items: AlignItems::Stretch,
            align_self: AlignSelf::Auto,
            justify_self: AlignSelf::Auto,
            gap: 0.0,
            gap_dimension: None,
            row_gap: None,
            column_gap: None,
            order: 0,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Dimension::Auto,
            font_size: Self::DEFAULT_FONT_SIZE,
            grid_placement: GridPlacement::default(),
            grid_auto_flow: GridAutoFlow::default(),
            scroll_container: false,
        }
    }
}

impl LayoutStyle {
    /// Resolves percentage padding against the containing inline size.
    pub fn resolve_padding(&mut self, containing_inline: f32) {
        if let Some(lengths) = &self.padding_lengths {
            let padding = lengths.resolve(containing_inline);
            self.padding = Insets {
                top: padding.top.max(0.0),
                right: padding.right.max(0.0),
                bottom: padding.bottom.max(0.0),
                left: padding.left.max(0.0),
            };
        }
    }

    /// Returns a copy with `display` replaced.
    #[must_use]
    pub const fn with_display(mut self, display: LayoutDisplay) -> Self {
        self.display = display;
        self
    }

    /// Returns a copy with `width` replaced.
    #[must_use]
    pub fn with_width(mut self, width: Dimension) -> Self {
        self.width = width;
        self
    }

    /// Returns a copy with `height` replaced.
    #[must_use]
    pub fn with_height(mut self, height: Dimension) -> Self {
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
    pub fn with_inset(mut self, inset: Inset) -> Self {
        self.inset = inset;
        self
    }

    #[doc = include_str!("../../docs/api/model/layout_style.doc/relative_offset.md")]
    #[must_use]
    pub fn relative_offset(&self, containing_width: f32, containing_height: f32) -> (f32, f32) {
        if !self.position.is_relative() {
            return (0.0, 0.0);
        }
        let dx = self
            .inset
            .left
            .resolve_signed(containing_width)
            .or_else(|| {
                self.inset
                    .right
                    .resolve_signed(containing_width)
                    .map(|right| -right)
            });
        let dy = self
            .inset
            .top
            .resolve_signed(containing_height)
            .or_else(|| {
                self.inset
                    .bottom
                    .resolve_signed(containing_height)
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
        AlignItems, AlignSelf, Dimension, FlexDirection, FlexWrap, Inset, JustifyContent,
        LayoutDisplay, MarginEdges, Position,
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
        assert_eq!(style.margin, MarginEdges::ZERO);
        assert_eq!(style.padding, Insets::uniform(0.0));
        assert_eq!(style.border, Insets::uniform(0.0));
        assert_eq!(style.flex_direction, FlexDirection::Row);
        assert_eq!(style.flex_wrap, FlexWrap::NoWrap);
        assert_eq!(style.justify_content, JustifyContent::Normal);
        assert_eq!(style.align_content, JustifyContent::Normal);
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
    fn absolute_and_fixed_have_no_in_flow_relative_offset() {
        // Out-of-flow schemes are placed by the coordinator's out-of-flow path,
        // not by the in-flow relative offset, so `relative_offset` stays `(0, 0)`.
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
