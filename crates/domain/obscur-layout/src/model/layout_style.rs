#![doc = include_str!("layout_style.doc/_module.md")]

use crate::algorithm::{GridAutoFlow, GridPlacement};
use crate::model::{
    AlignItems, AlignSelf, BoxSizing, Dimension, Direction, FlexDirection, FlexWrap, Inset,
    JustifyContent, LayoutDisplay, MarginEdges, Position, WritingMode,
};
use obscur_geometry::Insets;

#[doc = include_str!("layout_style.doc/LayoutStyle.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutStyle {
    #[doc = include_str!("layout_style.doc/display.md")]
    pub display: LayoutDisplay,

    #[doc = include_str!("layout_style.doc/direction.md")]
    pub direction: Direction,

    #[doc = include_str!("layout_style.doc/writing_mode.md")]
    pub writing_mode: WritingMode,

    #[doc = include_str!("layout_style.doc/position.md")]
    pub position: Position,

    #[doc = include_str!("layout_style.doc/inset.md")]
    pub inset: Inset,

    #[doc = include_str!("layout_style.doc/width.md")]
    pub width: Dimension,
    #[doc = include_str!("layout_style.doc/height.md")]
    pub height: Dimension,
    #[doc = include_str!("layout_style.doc/min_width.md")]
    pub min_width: Dimension,
    #[doc = include_str!("layout_style.doc/min_height.md")]
    pub min_height: Dimension,
    #[doc = include_str!("layout_style.doc/max_width.md")]
    pub max_width: Dimension,
    #[doc = include_str!("layout_style.doc/max_height.md")]
    pub max_height: Dimension,

    #[doc = include_str!("layout_style.doc/box_sizing.md")]
    pub box_sizing: BoxSizing,

    #[doc = include_str!("layout_style.doc/margin.md")]
    pub margin: MarginEdges,
    #[doc = include_str!("layout_style.doc/padding.md")]
    pub padding: Insets,
    #[doc = include_str!("layout_style.doc/border.md")]
    pub border: Insets,

    #[doc = include_str!("layout_style.doc/flex_direction.md")]
    pub flex_direction: FlexDirection,
    #[doc = include_str!("layout_style.doc/flex_wrap.md")]
    pub flex_wrap: FlexWrap,
    #[doc = include_str!("layout_style.doc/justify_content.md")]
    pub justify_content: JustifyContent,
    #[doc = include_str!("layout_style.doc/align_content.md")]
    pub align_content: JustifyContent,
    #[doc = include_str!("layout_style.doc/align_items.md")]
    pub align_items: AlignItems,
    #[doc = include_str!("layout_style.doc/justify_items.md")]
    pub justify_items: AlignItems,
    #[doc = include_str!("layout_style.doc/align_self.md")]
    pub align_self: AlignSelf,
    #[doc = include_str!("layout_style.doc/justify_self.md")]
    pub justify_self: AlignSelf,
    #[doc = include_str!("layout_style.doc/gap.md")]
    pub gap: f32,

    #[doc = include_str!("layout_style.doc/flex_grow.md")]
    pub flex_grow: f32,
    #[doc = include_str!("layout_style.doc/flex_shrink.md")]
    pub flex_shrink: f32,
    #[doc = include_str!("layout_style.doc/flex_basis.md")]
    pub flex_basis: Dimension,

    #[doc = include_str!("layout_style.doc/font_size.md")]
    pub font_size: f32,

    #[doc = include_str!("layout_style.doc/grid_placement.md")]
    pub grid_placement: GridPlacement,

    #[doc = include_str!("layout_style.doc/grid_auto_flow.md")]
    pub grid_auto_flow: GridAutoFlow,

    #[doc = include_str!("layout_style.doc/scroll_container.md")]
    pub scroll_container: bool,
}

impl LayoutStyle {
    #[doc = include_str!("layout_style.doc/DEFAULT_FONT_SIZE.md")]
    pub const DEFAULT_FONT_SIZE: f32 = 16.0;
}

impl Default for LayoutStyle {
    #[doc = include_str!("layout_style.doc/default.md")]
    fn default() -> Self {
        Self {
            display: LayoutDisplay::Block,
            direction: Direction::Ltr,
            writing_mode: WritingMode::HorizontalTb,
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
            border: Insets::uniform(0.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
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
            scroll_container: false,
        }
    }
}

impl LayoutStyle {
    #[doc = include_str!("layout_style.doc/with_display.md")]
    #[must_use]
    pub const fn with_display(mut self, display: LayoutDisplay) -> Self {
        self.display = display;
        self
    }

    #[doc = include_str!("layout_style.doc/with_width.md")]
    #[must_use]
    pub const fn with_width(mut self, width: Dimension) -> Self {
        self.width = width;
        self
    }

    #[doc = include_str!("layout_style.doc/with_height.md")]
    #[must_use]
    pub const fn with_height(mut self, height: Dimension) -> Self {
        self.height = height;
        self
    }

    #[doc = include_str!("layout_style.doc/with_font_size.md")]
    #[must_use]
    pub const fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    #[doc = include_str!("layout_style.doc/with_position.md")]
    #[must_use]
    pub const fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    #[doc = include_str!("layout_style.doc/with_inset.md")]
    #[must_use]
    pub const fn with_inset(mut self, inset: Inset) -> Self {
        self.inset = inset;
        self
    }

    #[doc = include_str!("layout_style.doc/relative_offset.md")]
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

    #[doc = include_str!("layout_style.doc/with_grid_placement.md")]
    #[must_use]
    pub const fn with_grid_placement(mut self, grid_placement: GridPlacement) -> Self {
        self.grid_placement = grid_placement;
        self
    }

    #[doc = include_str!("layout_style.doc/grid_placement_2.md")]
    #[must_use]
    pub const fn grid_placement(&self) -> GridPlacement {
        self.grid_placement
    }

    #[doc = include_str!("layout_style.doc/with_grid_auto_flow.md")]
    #[must_use]
    pub const fn with_grid_auto_flow(mut self, grid_auto_flow: GridAutoFlow) -> Self {
        self.grid_auto_flow = grid_auto_flow;
        self
    }

    #[doc = include_str!("layout_style.doc/grid_auto_flow_2.md")]
    #[must_use]
    pub const fn grid_auto_flow(&self) -> GridAutoFlow {
        self.grid_auto_flow
    }

    #[doc = include_str!("layout_style.doc/with_justify_items.md")]
    #[must_use]
    pub const fn with_justify_items(mut self, justify_items: AlignItems) -> Self {
        self.justify_items = justify_items;
        self
    }

    #[doc = include_str!("layout_style.doc/with_align_items.md")]
    #[must_use]
    pub const fn with_align_items(mut self, align_items: AlignItems) -> Self {
        self.align_items = align_items;
        self
    }

    #[doc = include_str!("layout_style.doc/with_align_self.md")]
    #[must_use]
    pub const fn with_align_self(mut self, align_self: AlignSelf) -> Self {
        self.align_self = align_self;
        self
    }

    #[doc = include_str!("layout_style.doc/with_justify_self.md")]
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
