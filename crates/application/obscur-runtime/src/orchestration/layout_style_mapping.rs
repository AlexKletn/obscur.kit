#![doc = include_str!("layout_style_mapping.doc/_module.md")]

use obscur_geometry::Size;
use obscur_identity::InternedSymbol;
use obscur_layout::algorithm::{GridAutoFlow, GridLine, GridPlacement};
use obscur_layout::model::{
    AlignItems, AlignSelf, BoxSizing, Direction, FlexDirection, FlexWrap, Inset, JustifyContent,
    LayoutDisplay, LayoutStyle, Position, WritingMode,
};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{GridLineValue, PropertyId, PropertyValue, layout_keyword};
use obscur_text::shaping::ParagraphShaper;

use super::length_resolution::{
    ch_width_for, dimension_of, font_size_px, insets_of, length_context, length_px_of,
    margin_edges_of,
};

#[doc = include_str!("layout_style_mapping.doc/layout_style_from.md")]
pub(super) fn layout_style_from(
    computed: &ComputedStyle,
    viewport: Size,
    root_font_size: f32,
    paragraph: Option<&ParagraphShaper>,
) -> LayoutStyle {
    let default = LayoutStyle::default();
    let ctx = length_context(
        computed,
        viewport,
        root_font_size,
        ch_width_for(computed, paragraph),
    );
    LayoutStyle {
        display: display_of(computed),
        // The box's inline base `direction`, so flow layout can settle an
        // over-constrained in-flow child against the correct physical edge
        // (line-left under `ltr`, line-right under `rtl`). Logical box-model
        // longhands were already folded to physical ones by the style crate;
        // this carries only the geometry that mapping cannot express. An
        // unauthored value stays `Ltr`, byte-for-byte identical to before.
        direction: direction_of(computed),
        // The box's `writing-mode`, so flow layout can stack its in-flow children
        // along the correct physical axis and origin. An unauthored value stays
        // `horizontal-tb`, byte-for-byte identical to before. Logical box-model
        // longhands were already folded to physical ones under this same value by
        // the style crate.
        writing_mode: writing_mode_of(computed),
        // Positioning scheme + insets. An unauthored `position` stays `Static`
        // and unauthored insets stay `Auto`, so a default node contributes no
        // offset — byte-for-byte identical to before positioning was wired.
        position: position_of(computed),
        inset: Inset::new(
            dimension_of(computed, PropertyId::Top, &ctx),
            dimension_of(computed, PropertyId::Right, &ctx),
            dimension_of(computed, PropertyId::Bottom, &ctx),
            dimension_of(computed, PropertyId::Left, &ctx),
        ),
        width: dimension_of(computed, PropertyId::Width, &ctx),
        height: dimension_of(computed, PropertyId::Height, &ctx),
        // `box-sizing`: an unauthored value stays `content-box` (the CSS default),
        // so a node that never sets it is byte-for-byte identical to before.
        box_sizing: box_sizing_of(computed),
        min_width: dimension_of(computed, PropertyId::MinWidth, &ctx),
        min_height: dimension_of(computed, PropertyId::MinHeight, &ctx),
        max_width: dimension_of(computed, PropertyId::MaxWidth, &ctx),
        max_height: dimension_of(computed, PropertyId::MaxHeight, &ctx),
        margin: margin_edges_of(
            computed,
            PropertyId::MarginTop,
            PropertyId::MarginRight,
            PropertyId::MarginBottom,
            PropertyId::MarginLeft,
            &ctx,
        ),
        padding: insets_of(
            computed,
            PropertyId::PaddingTop,
            PropertyId::PaddingRight,
            PropertyId::PaddingBottom,
            PropertyId::PaddingLeft,
            &ctx,
        ),
        // Border insets from the four `border-*-width` longhands. Unauthored
        // widths compute to `0` (their initial), so a node with no border keeps
        // the zero default — byte-for-byte identical to before borders existed.
        border: insets_of(
            computed,
            PropertyId::BorderTopWidth,
            PropertyId::BorderRightWidth,
            PropertyId::BorderBottomWidth,
            PropertyId::BorderLeftWidth,
            &ctx,
        ),
        flex_direction: flex_direction_of(computed),
        // `flex-wrap`: an unauthored value stays `NoWrap` (single-line), so a
        // container that never wraps is byte-for-byte identical to before.
        flex_wrap: flex_wrap_of(computed),
        justify_content: justify_content_of(computed),
        // Grid container: block-axis (row) track distribution. Unauthored stays
        // `FlexStart` (no distribution), so a default grid packs rows at the top.
        align_content: align_content_of(computed),
        align_items: align_items_of(computed),
        // Grid container item-alignment defaults + per-item overrides. Unauthored
        // values stay `Stretch` / `Auto`, so a default grid fills each cell
        // exactly as before.
        justify_items: justify_items_of(computed),
        align_self: align_self_of(computed, PropertyId::AlignSelf),
        justify_self: align_self_of(computed, PropertyId::JustifySelf),
        gap: length_px_of(computed, PropertyId::Gap, &ctx),
        flex_grow: number_of(computed, PropertyId::FlexGrow, default.flex_grow),
        flex_shrink: number_of(computed, PropertyId::FlexShrink, default.flex_shrink),
        flex_basis: dimension_of(computed, PropertyId::FlexBasis, &ctx),
        // The node's font size, so a text leaf can shape and wrap to its
        // available width inside the constraint-aware measure/arrange pass.
        font_size: font_size_px(computed),
        // Line-based grid placement from the four `grid-column`/`grid-row`
        // longhands; unauthored edges stay `Auto`, so a non-placed item keeps the
        // all-auto default and auto-places exactly as before.
        grid_placement: grid_placement_of(computed),
        // The container's `grid-auto-flow`; an unauthored value stays `Row`
        // (sparse), so auto-placement is byte-identical to before.
        grid_auto_flow: grid_auto_flow_of(computed),
        // Whether this box scrolls its overflow (`scroll`/`auto`/`hidden`), which
        // css-flexbox-1 §4.5 uses to zero a flex item's automatic minimum size.
        // Unauthored `overflow` is `visible`, so a non-scrolling box stays `false`.
        scroll_container: scroll_container_of(computed),
    }
}

#[doc = include_str!("layout_style_mapping.doc/scroll_container_of.md")]
fn scroll_container_of(computed: &ComputedStyle) -> bool {
    matches!(
        computed
            .get(PropertyId::Overflow)
            .map(ComputedValue::value),
        Some(&PropertyValue::Keyword(sym))
            if sym == layout_keyword::OVERFLOW_SCROLL
                || sym == layout_keyword::OVERFLOW_AUTO
                || sym == layout_keyword::OVERFLOW_HIDDEN
    )
}

#[doc = include_str!("layout_style_mapping.doc/grid_auto_flow_of.md")]
fn grid_auto_flow_of(computed: &ComputedStyle) -> GridAutoFlow {
    match keyword_sym(computed, PropertyId::GridAutoFlow) {
        Some(sym) if sym == layout_keyword::GRID_AUTO_FLOW_COLUMN => GridAutoFlow::Column,
        Some(sym) if sym == layout_keyword::GRID_AUTO_FLOW_ROW_DENSE => GridAutoFlow::RowDense,
        Some(sym) if sym == layout_keyword::GRID_AUTO_FLOW_COLUMN_DENSE => {
            GridAutoFlow::ColumnDense
        }
        _ => GridAutoFlow::Row,
    }
}

#[doc = include_str!("layout_style_mapping.doc/grid_placement_of.md")]
fn grid_placement_of(computed: &ComputedStyle) -> GridPlacement {
    GridPlacement::new(
        grid_line_of(computed, PropertyId::GridColumnStart),
        grid_line_of(computed, PropertyId::GridColumnEnd),
        grid_line_of(computed, PropertyId::GridRowStart),
        grid_line_of(computed, PropertyId::GridRowEnd),
    )
}

#[doc = include_str!("layout_style_mapping.doc/grid_line_of.md")]
fn grid_line_of(computed: &ComputedStyle, property: PropertyId) -> GridLine {
    match computed
        .get(property)
        .map(ComputedValue::value)
        .and_then(PropertyValue::as_grid_line)
    {
        Some(GridLineValue::Line(index)) => GridLine::Line(*index),
        Some(GridLineValue::Span(n)) => GridLine::Span(*n),
        Some(GridLineValue::Named(_) | GridLineValue::Auto) | None => GridLine::Auto,
    }
}

#[doc = include_str!("layout_style_mapping.doc/number_of.md")]
fn number_of(computed: &ComputedStyle, property: PropertyId, default: f32) -> f32 {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Number(value)) => value,
        _ => default,
    }
}

#[doc = include_str!("layout_style_mapping.doc/keyword_sym.md")]
fn keyword_sym(computed: &ComputedStyle, property: PropertyId) -> Option<InternedSymbol> {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Keyword(sym)) => Some(sym),
        _ => None,
    }
}

#[doc = include_str!("layout_style_mapping.doc/display_of.md")]
fn display_of(computed: &ComputedStyle) -> LayoutDisplay {
    match keyword_sym(computed, PropertyId::Display) {
        Some(sym) if sym == layout_keyword::DISPLAY_FLEX => LayoutDisplay::Flex,
        Some(sym) if sym == layout_keyword::DISPLAY_GRID => LayoutDisplay::Grid,
        Some(sym) if sym == layout_keyword::DISPLAY_NONE => LayoutDisplay::None,
        Some(sym) if sym == layout_keyword::DISPLAY_CONTENTS => LayoutDisplay::Contents,
        _ => LayoutDisplay::Block,
    }
}

#[doc = include_str!("layout_style_mapping.doc/box_sizing_of.md")]
fn box_sizing_of(computed: &ComputedStyle) -> BoxSizing {
    match keyword_sym(computed, PropertyId::BoxSizing) {
        Some(sym) if sym == layout_keyword::BOX_SIZING_BORDER_BOX => BoxSizing::BorderBox,
        _ => BoxSizing::ContentBox,
    }
}

#[doc = include_str!("layout_style_mapping.doc/direction_of.md")]
fn direction_of(computed: &ComputedStyle) -> Direction {
    match keyword_sym(computed, PropertyId::Direction) {
        Some(sym) if sym == layout_keyword::DIRECTION_RTL => Direction::Rtl,
        _ => Direction::Ltr,
    }
}

#[doc = include_str!("layout_style_mapping.doc/writing_mode_of.md")]
fn writing_mode_of(computed: &ComputedStyle) -> WritingMode {
    match keyword_sym(computed, PropertyId::WritingMode) {
        Some(sym) if sym == layout_keyword::WRITING_MODE_VERTICAL_RL => WritingMode::VerticalRl,
        Some(sym) if sym == layout_keyword::WRITING_MODE_VERTICAL_LR => WritingMode::VerticalLr,
        _ => WritingMode::HorizontalTb,
    }
}

#[doc = include_str!("layout_style_mapping.doc/position_of.md")]
fn position_of(computed: &ComputedStyle) -> Position {
    match keyword_sym(computed, PropertyId::Position) {
        Some(sym) if sym == layout_keyword::POSITION_RELATIVE => Position::Relative,
        Some(sym) if sym == layout_keyword::POSITION_ABSOLUTE => Position::Absolute,
        Some(sym) if sym == layout_keyword::POSITION_FIXED => Position::Fixed,
        Some(sym) if sym == layout_keyword::POSITION_STICKY => Position::Sticky,
        _ => Position::Static,
    }
}

#[doc = include_str!("layout_style_mapping.doc/flex_direction_of.md")]
fn flex_direction_of(computed: &ComputedStyle) -> FlexDirection {
    match keyword_sym(computed, PropertyId::FlexDirection) {
        Some(sym) if sym == layout_keyword::FLEX_ROW_REVERSE => FlexDirection::RowReverse,
        Some(sym) if sym == layout_keyword::FLEX_COLUMN => FlexDirection::Column,
        Some(sym) if sym == layout_keyword::FLEX_COLUMN_REVERSE => FlexDirection::ColumnReverse,
        _ => FlexDirection::Row,
    }
}

#[doc = include_str!("layout_style_mapping.doc/justify_content_keyword.md")]
fn justify_content_keyword(sym: Option<InternedSymbol>) -> JustifyContent {
    match sym {
        Some(sym) if sym == layout_keyword::JUSTIFY_CENTER => JustifyContent::Center,
        Some(sym) if sym == layout_keyword::JUSTIFY_END => JustifyContent::FlexEnd,
        Some(sym) if sym == layout_keyword::JUSTIFY_SPACE_BETWEEN => JustifyContent::SpaceBetween,
        Some(sym) if sym == layout_keyword::JUSTIFY_SPACE_AROUND => JustifyContent::SpaceAround,
        Some(sym) if sym == layout_keyword::JUSTIFY_SPACE_EVENLY => JustifyContent::SpaceEvenly,
        Some(sym) if sym == layout_keyword::JUSTIFY_STRETCH => JustifyContent::Stretch,
        _ => JustifyContent::FlexStart,
    }
}

#[doc = include_str!("layout_style_mapping.doc/justify_content_of.md")]
fn justify_content_of(computed: &ComputedStyle) -> JustifyContent {
    justify_content_keyword(keyword_sym(computed, PropertyId::JustifyContent))
}

#[doc = include_str!("layout_style_mapping.doc/align_content_of.md")]
fn align_content_of(computed: &ComputedStyle) -> JustifyContent {
    justify_content_keyword(keyword_sym(computed, PropertyId::AlignContent))
}

#[doc = include_str!("layout_style_mapping.doc/flex_wrap_of.md")]
fn flex_wrap_of(computed: &ComputedStyle) -> FlexWrap {
    match keyword_sym(computed, PropertyId::FlexWrap) {
        Some(sym) if sym == layout_keyword::FLEX_WRAP_WRAP => FlexWrap::Wrap,
        Some(sym) if sym == layout_keyword::FLEX_WRAP_WRAP_REVERSE => FlexWrap::WrapReverse,
        _ => FlexWrap::NoWrap,
    }
}

#[doc = include_str!("layout_style_mapping.doc/align_items_of.md")]
fn align_items_of(computed: &ComputedStyle) -> AlignItems {
    match keyword_sym(computed, PropertyId::AlignItems) {
        Some(sym) if sym == layout_keyword::ALIGN_START => AlignItems::FlexStart,
        Some(sym) if sym == layout_keyword::ALIGN_CENTER => AlignItems::Center,
        Some(sym) if sym == layout_keyword::ALIGN_END => AlignItems::FlexEnd,
        _ => AlignItems::Stretch,
    }
}

#[doc = include_str!("layout_style_mapping.doc/justify_items_of.md")]
fn justify_items_of(computed: &ComputedStyle) -> AlignItems {
    match keyword_sym(computed, PropertyId::JustifyItems) {
        Some(sym) if sym == layout_keyword::ALIGN_START => AlignItems::Start,
        Some(sym) if sym == layout_keyword::ALIGN_CENTER => AlignItems::Center,
        Some(sym) if sym == layout_keyword::ALIGN_END => AlignItems::End,
        _ => AlignItems::Stretch,
    }
}

#[doc = include_str!("layout_style_mapping.doc/align_self_of.md")]
fn align_self_of(computed: &ComputedStyle, property: PropertyId) -> AlignSelf {
    match keyword_sym(computed, property) {
        Some(sym) if sym == layout_keyword::ALIGN_START => AlignSelf::Start,
        Some(sym) if sym == layout_keyword::ALIGN_CENTER => AlignSelf::Center,
        Some(sym) if sym == layout_keyword::ALIGN_END => AlignSelf::End,
        Some(sym) if sym == layout_keyword::ALIGN_STRETCH => AlignSelf::Stretch,
        _ => AlignSelf::Auto,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Insets;
    use obscur_identity::NodeId;
    use obscur_layout::model::{Dimension, MarginEdges};
    use obscur_style::property::Length;

    use crate::orchestration::GridTemplates;
    use crate::orchestration::formatting_context::algorithm_for;

    /// A default resolution viewport for the mapping tests.
    const TEST_VIEWPORT: Size = Size::new(800.0, 600.0);
    /// A default root font size for the mapping tests.
    const TEST_ROOT_FONT: f32 = 16.0;

    #[test]
    fn mapping_reads_width_and_defaults_display_to_block() {
        let mut computed = ComputedStyle::new();
        computed.set(PropertyId::Width, ComputedValue::length(Length::px(120.0)));
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.width, Dimension::Px(120.0));
        // Height absent -> Auto.
        assert_eq!(style.height, Dimension::Auto);
        // Display is always Block in Phase 1 (interner not available).
        assert_eq!(style.display, LayoutDisplay::Block);
    }

    #[test]
    fn mapping_percent_and_auto() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Width,
            ComputedValue::length(Length::Percent(50.0)),
        );
        computed.set(PropertyId::Height, ComputedValue::length(Length::Auto));
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.width, Dimension::Percent(50.0));
        assert_eq!(style.height, Dimension::Auto);
    }

    #[test]
    fn mapping_resolves_relative_units_against_context() {
        use obscur_style::property::LengthExpr;
        let mut computed = ComputedStyle::new();
        // font-size 16px -> em resolves against it; viewport 800x600 -> vw/vh.
        computed.set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(16.0)),
        );
        computed.set(PropertyId::Width, ComputedValue::length(Length::Em(3.0)));
        computed.set(PropertyId::Height, ComputedValue::length(Length::Vw(5.0)));
        // rem uses the document root font size (20), not the node font size.
        computed.set(
            PropertyId::MinWidth,
            ComputedValue::length(Length::Rem(2.0)),
        );
        // calc(10px + 2em) = 42.
        computed.set(
            PropertyId::MaxWidth,
            ComputedValue::calc(LengthExpr::Sum(
                Box::new(LengthExpr::Leaf(Length::px(10.0))),
                Box::new(LengthExpr::Leaf(Length::Em(2.0))),
            )),
        );
        let style = layout_style_from(&computed, TEST_VIEWPORT, 20.0, None);
        assert_eq!(style.width, Dimension::Px(48.0)); // 3em
        assert_eq!(style.height, Dimension::Px(40.0)); // 5vw of 800
        assert_eq!(style.min_width, Dimension::Px(40.0)); // 2rem of 20
        assert_eq!(style.max_width, Dimension::Px(42.0)); // calc(10px + 2em)
    }

    #[test]
    fn mapping_resolves_relative_gap_to_pixels() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(10.0)),
        );
        computed.set(PropertyId::Gap, ComputedValue::length(Length::Em(2.0)));
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.gap, 20.0); // 2em of 10px
    }

    #[test]
    fn mapping_reads_position_and_insets() {
        use obscur_style::property::layout_keyword;
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Position,
            ComputedValue::from_concrete(PropertyValue::Keyword(layout_keyword::POSITION_RELATIVE))
                .expect("concrete"),
        );
        computed.set(PropertyId::Top, ComputedValue::length(Length::px(10.0)));
        computed.set(PropertyId::Left, ComputedValue::length(Length::px(5.0)));
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.position, Position::Relative);
        assert_eq!(style.inset.top, Dimension::Px(10.0));
        assert_eq!(style.inset.left, Dimension::Px(5.0));
        // Unauthored edges default to `Auto`.
        assert_eq!(style.inset.right, Dimension::Auto);
        assert_eq!(style.inset.bottom, Dimension::Auto);
    }

    #[test]
    fn mapping_defaults_position_to_static() {
        // A node with no authored `position` stays `Static` with all-`auto` insets.
        let computed = ComputedStyle::new();
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.position, Position::Static);
        assert_eq!(style.inset, Inset::AUTO);
    }

    #[test]
    fn mapping_builds_grid_placement_from_computed_lines() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::GridColumnStart,
            ComputedValue::from_concrete(PropertyValue::GridLine(GridLineValue::Line(1)))
                .expect("concrete"),
        );
        computed.set(
            PropertyId::GridColumnEnd,
            ComputedValue::from_concrete(PropertyValue::GridLine(GridLineValue::Line(3)))
                .expect("concrete"),
        );
        computed.set(
            PropertyId::GridRowStart,
            ComputedValue::from_concrete(PropertyValue::GridLine(GridLineValue::Span(2)))
                .expect("concrete"),
        );
        // `grid-row-end` left unset -> Auto.
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        let placement = style.grid_placement();
        assert_eq!(placement.column_start(), GridLine::Line(1));
        assert_eq!(placement.column_end(), GridLine::Line(3));
        assert_eq!(placement.row_start(), GridLine::Span(2));
        assert_eq!(placement.row_end(), GridLine::Auto);
        // The column axis resolves to a definite start of track 0, span 2.
        assert_eq!(placement.resolve_column(3), (Some(0), 2));
    }

    #[test]
    fn mapping_defaults_grid_placement_to_all_auto() {
        // A node with no authored line placement keeps the all-Auto default, so it
        // auto-places exactly as before.
        let computed = ComputedStyle::new();
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.grid_placement(), GridPlacement::default());
    }

    #[test]
    fn mapping_reads_grid_auto_flow_keyword() {
        // Each reserved `grid-auto-flow` handle maps to its flow mode.
        for (sym, expected) in [
            (layout_keyword::GRID_AUTO_FLOW_COLUMN, GridAutoFlow::Column),
            (
                layout_keyword::GRID_AUTO_FLOW_ROW_DENSE,
                GridAutoFlow::RowDense,
            ),
            (
                layout_keyword::GRID_AUTO_FLOW_COLUMN_DENSE,
                GridAutoFlow::ColumnDense,
            ),
            (layout_keyword::GRID_AUTO_FLOW_ROW, GridAutoFlow::Row),
        ] {
            let mut computed = ComputedStyle::new();
            computed.set(PropertyId::GridAutoFlow, ComputedValue::keyword(sym));
            let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
            assert_eq!(style.grid_auto_flow(), expected);
        }
        // Absent -> the `Row` (sparse) default.
        let computed = ComputedStyle::new();
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.grid_auto_flow(), GridAutoFlow::Row);
    }

    #[test]
    fn mapping_reads_grid_item_alignment_keywords() {
        // Container defaults: `justify-items` / `align-items`.
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::JustifyItems,
            ComputedValue::keyword(layout_keyword::ALIGN_CENTER),
        );
        computed.set(
            PropertyId::AlignItems,
            ComputedValue::keyword(layout_keyword::ALIGN_END),
        );
        // Per-item overrides: `justify-self` / `align-self` (incl. `auto`).
        computed.set(
            PropertyId::JustifySelf,
            ComputedValue::keyword(layout_keyword::ALIGN_START),
        );
        computed.set(
            PropertyId::AlignSelf,
            ComputedValue::keyword(layout_keyword::ALIGN_AUTO),
        );
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.justify_items, AlignItems::Center);
        assert_eq!(style.align_items, AlignItems::FlexEnd);
        assert_eq!(style.justify_self, AlignSelf::Start);
        assert_eq!(style.align_self, AlignSelf::Auto);

        // Absent -> the layout defaults (`Stretch` container, `Auto` item).
        let empty = ComputedStyle::new();
        let default_style = layout_style_from(&empty, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(default_style.justify_items, AlignItems::Stretch);
        assert_eq!(default_style.justify_self, AlignSelf::Auto);
        assert_eq!(default_style.align_self, AlignSelf::Auto);
    }

    #[test]
    fn mapping_populates_every_field_from_computed_style() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_FLEX),
        );
        // Box model.
        computed.set(
            PropertyId::MarginTop,
            ComputedValue::length(Length::px(1.0)),
        );
        computed.set(
            PropertyId::MarginRight,
            ComputedValue::length(Length::px(2.0)),
        );
        computed.set(
            PropertyId::MarginBottom,
            ComputedValue::length(Length::px(3.0)),
        );
        computed.set(
            PropertyId::MarginLeft,
            ComputedValue::length(Length::px(4.0)),
        );
        computed.set(
            PropertyId::PaddingTop,
            ComputedValue::length(Length::px(5.0)),
        );
        computed.set(
            PropertyId::PaddingRight,
            ComputedValue::length(Length::px(6.0)),
        );
        computed.set(
            PropertyId::PaddingBottom,
            ComputedValue::length(Length::px(7.0)),
        );
        computed.set(
            PropertyId::PaddingLeft,
            ComputedValue::length(Length::px(8.0)),
        );
        // Sizing constraints.
        computed.set(
            PropertyId::MinWidth,
            ComputedValue::length(Length::px(10.0)),
        );
        computed.set(
            PropertyId::MinHeight,
            ComputedValue::length(Length::px(11.0)),
        );
        computed.set(
            PropertyId::MaxWidth,
            ComputedValue::length(Length::Percent(80.0)),
        );
        computed.set(
            PropertyId::MaxHeight,
            ComputedValue::length(Length::px(13.0)),
        );
        // Flex container + item.
        computed.set(PropertyId::Gap, ComputedValue::length(Length::px(9.0)));
        computed.set(
            PropertyId::FlexDirection,
            ComputedValue::keyword(layout_keyword::FLEX_COLUMN),
        );
        computed.set(
            PropertyId::FlexWrap,
            ComputedValue::keyword(layout_keyword::FLEX_WRAP_WRAP),
        );
        computed.set(
            PropertyId::JustifyContent,
            ComputedValue::keyword(layout_keyword::JUSTIFY_CENTER),
        );
        computed.set(
            PropertyId::AlignItems,
            ComputedValue::keyword(layout_keyword::ALIGN_STRETCH),
        );
        computed.set(PropertyId::FlexGrow, ComputedValue::number(1.0));
        computed.set(PropertyId::FlexShrink, ComputedValue::number(2.0));
        computed.set(
            PropertyId::FlexBasis,
            ComputedValue::length(Length::px(50.0)),
        );

        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);

        assert_eq!(style.display, LayoutDisplay::Flex);
        assert_eq!(style.margin, MarginEdges::px(1.0, 2.0, 3.0, 4.0));
        assert_eq!(
            style.padding,
            Insets {
                top: 5.0,
                right: 6.0,
                bottom: 7.0,
                left: 8.0
            }
        );
        assert_eq!(style.min_width, Dimension::Px(10.0));
        assert_eq!(style.min_height, Dimension::Px(11.0));
        assert_eq!(style.max_width, Dimension::Percent(80.0));
        assert_eq!(style.max_height, Dimension::Px(13.0));
        assert_eq!(style.gap, 9.0);
        assert_eq!(style.flex_direction, FlexDirection::Column);
        assert_eq!(style.flex_wrap, FlexWrap::Wrap);
        assert_eq!(style.justify_content, JustifyContent::Center);
        assert_eq!(style.align_items, AlignItems::Stretch);
        assert_eq!(style.flex_grow, 1.0);
        assert_eq!(style.flex_shrink, 2.0);
        assert_eq!(style.flex_basis, Dimension::Px(50.0));
    }

    #[test]
    fn mapping_display_grid_maps_to_grid_and_has_algorithm() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_GRID),
        );
        let style = layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None);
        assert_eq!(style.display, LayoutDisplay::Grid);
        assert!(
            algorithm_for(
                LayoutDisplay::Grid,
                NodeId::from_raw(1),
                &GridTemplates::new(),
                0.0
            )
            .is_some()
        );
    }

    #[test]
    fn mapping_display_none_maps_to_none() {
        let mut computed = ComputedStyle::new();
        computed.set(
            PropertyId::Display,
            ComputedValue::keyword(layout_keyword::DISPLAY_NONE),
        );
        assert_eq!(
            layout_style_from(&computed, TEST_VIEWPORT, TEST_ROOT_FONT, None).display,
            LayoutDisplay::None
        );
    }
}
