//! `TaffyStyleMapper`: translates an obscur [`LayoutStyle`] into a `taffy::Style`.
//!
//! This is the internal, spec-sanctioned Taffy provider (spec §3.6: "Taffy
//! integration должна оставаться внутренней"). Taffy is a pure layout algorithm
//! library; here it is used purely as an implementation detail.
//!
//! ## §3.6 boundary
//!
//! The single method that produces a `taffy::Style` — [`TaffyStyleMapper::to_taffy`]
//! — is `pub(crate)`, **not** `pub`. That keeps `taffy::Style` out of the crate's
//! public API surface: no public signature in `obscur-layout` ever mentions a
//! `taffy::` type. The struct itself is `pub` (re-exported) so callers can name and
//! construct it, but only the crate-internal orchestrator ([`super::TaffyTreeView`])
//! can invoke the bridge.

// The mapper is a stateless value object whose methods are spec-named to take
// `&self` (spec §3.6 bridge). Clippy's self-convention/`unused_self` pedantic lints
// do not apply to this deliberate shape.
#![allow(
    clippy::unused_self,
    clippy::trivially_copy_pass_by_ref,
    clippy::wrong_self_convention
)]

use crate::model::{
    AlignItems, Dimension, FlexDirection, FlexWrap, JustifyContent, LayoutDisplay, LayoutStyle,
    Margin, MarginEdges,
};
use obscur_geometry::Insets;

/// Maps an obscur [`LayoutStyle`] to the equivalent `taffy::Style`.
///
/// Stateless: construct with [`TaffyStyleMapper::new`] (or `Default`) and reuse.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TaffyStyleMapper;

impl TaffyStyleMapper {
    /// Creates a new, stateless style mapper.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Translates an obscur [`LayoutStyle`] into a `taffy::Style`.
    ///
    /// Every field obscur models is mapped explicitly; all remaining `taffy::Style`
    /// fields (grid, positioning, overflow, ...) are taken from `taffy::Style::default()`.
    ///
    /// ## Percent convention
    ///
    /// obscur's [`Dimension::Percent`] is expressed in `0..=100` units (`50.0` means
    /// 50%), whereas taffy's percent is `0..=1`. Percentages are therefore divided by
    /// `100.0` on the way across.
    ///
    /// ## §3.6
    ///
    /// This method is `pub(crate)`: `taffy::Style` must never appear in the crate's
    /// public API. It is the internal glue consumed only by [`super::TaffyTreeView`].
    #[must_use]
    pub(crate) fn to_taffy(&self, style: &LayoutStyle) -> taffy::Style {
        taffy::Style {
            display: Self::display(style.display),
            size: taffy::Size {
                width: self.dimension(style.width),
                height: self.dimension(style.height),
            },
            min_size: taffy::Size {
                width: self.dimension(style.min_width),
                height: self.dimension(style.min_height),
            },
            max_size: taffy::Size {
                width: self.dimension(style.max_width),
                height: self.dimension(style.max_height),
            },
            margin: self.margin_lpa(style.margin),
            padding: self.insets_lp(style.padding),
            border: self.insets_lp(style.border),
            flex_direction: Self::flex_direction(style.flex_direction),
            flex_wrap: Self::flex_wrap(style.flex_wrap),
            justify_content: Some(Self::justify_content(style.justify_content)),
            align_items: Some(Self::align_items(style.align_items)),
            gap: taffy::Size {
                width: taffy::LengthPercentage::Length(style.gap),
                height: taffy::LengthPercentage::Length(style.gap),
            },
            flex_grow: style.flex_grow,
            flex_shrink: style.flex_shrink,
            flex_basis: self.dimension(style.flex_basis),
            ..taffy::Style::default()
        }
    }

    /// Maps an obscur [`Dimension`] to a `taffy::Dimension`.
    ///
    /// `Auto → Auto`, `Px(v) → Length(v)`, `Percent(p) → Percent(p / 100.0)`
    /// (see the percent convention on [`to_taffy`](Self::to_taffy)).
    ///
    /// # The intrinsic keywords degrade to `auto` here
    ///
    /// `taffy::Dimension` models exactly `Auto`/`Length`/`Percent` — it has no way
    /// to *say* `min-content`, `max-content` or `fit-content` on a sizing
    /// property, so there is nothing faithful to map them to and this returns
    /// `Auto`. That is a real behaviour difference from the native algorithms,
    /// which measure them: through this adapter, `width: max-content` sizes as
    /// `width: auto`.
    ///
    /// It is reported rather than papered over: the alternative — silently
    /// emitting some fixed length — would be a wrong number instead of a wrong
    /// keyword, and `auto` is at least the value CSS falls back to when a sizing
    /// keyword is unsupported. A faithful mapping needs taffy's own intrinsic
    /// sizing seam (its `AvailableSpace` measure), which this style-only mapper
    /// does not reach.
    fn dimension(&self, dimension: Dimension) -> taffy::Dimension {
        match dimension {
            Dimension::Auto => taffy::Dimension::Auto,
            Dimension::Px(value) => taffy::Dimension::Length(value),
            Dimension::Percent(percent) => taffy::Dimension::Percent(percent / 100.0),
            // See the note above: unrepresentable in `taffy::Dimension`.
            Dimension::MinContent | Dimension::MaxContent | Dimension::FitContent => {
                taffy::Dimension::Auto
            }
        }
    }

    /// Maps obscur [`MarginEdges`] to a `taffy::Rect<LengthPercentageAuto>`.
    ///
    /// Unlike the intrinsic sizing keywords — which `taffy::Dimension` cannot say
    /// at all, see [`dimension`](Self::dimension) — every margin maps exactly.
    /// `LengthPercentageAuto` is the same three-way choice with the same meanings:
    /// taffy distributes leftover space to `Auto` and resolves `Percent` against
    /// the containing block, so `margin: 0 auto` centres through this adapter as it
    /// does natively. Nothing here needs a caveat.
    fn margin_lpa(&self, margin: MarginEdges) -> taffy::Rect<taffy::LengthPercentageAuto> {
        let edge = |m: Margin| match m {
            Margin::Px(value) => taffy::LengthPercentageAuto::Length(value),
            // Percent convention: obscur is 0..=100, taffy 0..=1 (see `to_taffy`).
            Margin::Percent(percent) => taffy::LengthPercentageAuto::Percent(percent / 100.0),
            Margin::Auto => taffy::LengthPercentageAuto::Auto,
        };
        taffy::Rect {
            left: edge(margin.left),
            right: edge(margin.right),
            top: edge(margin.top),
            bottom: edge(margin.bottom),
        }
    }

    /// Maps obscur [`Insets`] to a `taffy::Rect<LengthPercentage>` (used for padding/border).
    fn insets_lp(&self, insets: Insets) -> taffy::Rect<taffy::LengthPercentage> {
        taffy::Rect {
            left: taffy::LengthPercentage::Length(insets.left),
            right: taffy::LengthPercentage::Length(insets.right),
            top: taffy::LengthPercentage::Length(insets.top),
            bottom: taffy::LengthPercentage::Length(insets.bottom),
        }
    }

    /// Maps [`LayoutDisplay`] to `taffy::Display`.
    fn display(display: LayoutDisplay) -> taffy::Display {
        match display {
            LayoutDisplay::Block => taffy::Display::Block,
            LayoutDisplay::Flex => taffy::Display::Flex,
            LayoutDisplay::Grid => taffy::Display::Grid,
            LayoutDisplay::None => taffy::Display::None,
        }
    }

    /// Maps [`FlexDirection`] to `taffy::FlexDirection` (1:1).
    fn flex_direction(direction: FlexDirection) -> taffy::FlexDirection {
        match direction {
            FlexDirection::Row => taffy::FlexDirection::Row,
            FlexDirection::Column => taffy::FlexDirection::Column,
            FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
            FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
        }
    }

    /// Maps [`FlexWrap`] to `taffy::FlexWrap` (1:1).
    fn flex_wrap(wrap: FlexWrap) -> taffy::FlexWrap {
        match wrap {
            FlexWrap::NoWrap => taffy::FlexWrap::NoWrap,
            FlexWrap::Wrap => taffy::FlexWrap::Wrap,
            FlexWrap::WrapReverse => taffy::FlexWrap::WrapReverse,
        }
    }

    /// Maps [`JustifyContent`] to `taffy::JustifyContent` (1:1).
    fn justify_content(justify: JustifyContent) -> taffy::JustifyContent {
        match justify {
            JustifyContent::FlexStart => taffy::JustifyContent::FlexStart,
            JustifyContent::FlexEnd => taffy::JustifyContent::FlexEnd,
            JustifyContent::Center => taffy::JustifyContent::Center,
            JustifyContent::SpaceBetween => taffy::JustifyContent::SpaceBetween,
            JustifyContent::SpaceAround => taffy::JustifyContent::SpaceAround,
            JustifyContent::SpaceEvenly => taffy::JustifyContent::SpaceEvenly,
        }
    }

    /// Maps [`AlignItems`] to `taffy::AlignItems` (1:1).
    fn align_items(align: AlignItems) -> taffy::AlignItems {
        match align {
            AlignItems::FlexStart => taffy::AlignItems::FlexStart,
            AlignItems::FlexEnd => taffy::AlignItems::FlexEnd,
            AlignItems::Start => taffy::AlignItems::Start,
            AlignItems::End => taffy::AlignItems::End,
            AlignItems::Center => taffy::AlignItems::Center,
            AlignItems::Stretch => taffy::AlignItems::Stretch,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TaffyStyleMapper;
    use crate::model::{
        AlignItems, Dimension, FlexDirection, JustifyContent, LayoutDisplay, LayoutStyle,
    };

    #[test]
    fn maps_display_flex() {
        let style = LayoutStyle::default().with_display(LayoutDisplay::Flex);
        let taffy = TaffyStyleMapper::new().to_taffy(&style);
        assert_eq!(taffy.display, taffy::Display::Flex);
    }

    #[test]
    fn maps_px_to_length() {
        let style = LayoutStyle::default().with_width(Dimension::Px(120.0));
        let taffy = TaffyStyleMapper::new().to_taffy(&style);
        assert_eq!(taffy.size.width, taffy::Dimension::Length(120.0));
    }

    #[test]
    fn maps_percent_dividing_by_100() {
        let style = LayoutStyle::default().with_height(Dimension::Percent(50.0));
        let taffy = TaffyStyleMapper::new().to_taffy(&style);
        assert_eq!(taffy.size.height, taffy::Dimension::Percent(0.5));
    }

    #[test]
    fn maps_auto_dimension() {
        let taffy = TaffyStyleMapper::new().to_taffy(&LayoutStyle::default());
        assert_eq!(taffy.size.width, taffy::Dimension::Auto);
    }

    #[test]
    fn maps_flex_direction_justify_align() {
        let style = LayoutStyle {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..LayoutStyle::default()
        };
        let taffy = TaffyStyleMapper::new().to_taffy(&style);
        assert_eq!(taffy.flex_direction, taffy::FlexDirection::Column);
        assert_eq!(
            taffy.justify_content,
            Some(taffy::JustifyContent::SpaceBetween)
        );
        assert_eq!(taffy.align_items, Some(taffy::AlignItems::Center));
    }

    #[test]
    fn maps_gap_on_both_axes() {
        let style = LayoutStyle {
            gap: 8.0,
            ..LayoutStyle::default()
        };
        let taffy = TaffyStyleMapper::new().to_taffy(&style);
        assert_eq!(taffy.gap.width, taffy::LengthPercentage::Length(8.0));
        assert_eq!(taffy.gap.height, taffy::LengthPercentage::Length(8.0));
    }
}
