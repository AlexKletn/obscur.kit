use std::rc::Rc;
use std::sync::Arc;

use obscur_identity::{FastMap, InternedSymbol, NodeId};

use crate::computed::{
    ComputedStyle, CustomValueParser, OverflowAxes, absolute_font_size, absolutize_value,
    initial_value_of, substitute, substituted_value,
};
use crate::property::{
    Length, LengthContext, LengthExpr, PropertyId, PropertyValue, WritingMode, layout_keyword,
};

use super::custom_range_value::CustomRangeValue;
use super::style_container::StyleContainer;
use super::{QueryContainer, StylePropertyValue, StyleRangeValue};

/// Snapshot of measured query containers and the logical ancestor links used to select them.
#[derive(Debug, Default, PartialEq)]
pub struct ContainerQueryEnvironment {
    containers: FastMap<NodeId, QueryContainer>,
    parents: FastMap<NodeId, NodeId>,
    styles: FastMap<NodeId, StyleContainer>,
    aliases: FastMap<NodeId, Arc<[Box<str>]>>,
}

impl ContainerQueryEnvironment {
    /// Registers measured axes for the named node.
    pub fn insert(&mut self, node: NodeId, container: QueryContainer) {
        self.containers.insert(node, container);
    }

    /// Records an eligible style container and its shared custom property map.
    pub fn insert_style(
        &mut self,
        node: NodeId,
        name: Option<Box<str>>,
        properties: Rc<FastMap<InternedSymbol, PropertyValue>>,
    ) {
        self.styles.insert(
            node,
            StyleContainer {
                name,
                properties,
                values: FastMap::default(),
                ranges: FastMap::default(),
                range_context: None,
                font_size_context: None,
                references: FastMap::default(),
            },
        );
    }

    /// Records a style container with the computed longhands requested by active queries.
    pub fn insert_computed_style(
        &mut self,
        node: NodeId,
        name: Option<Box<str>>,
        computed: &ComputedStyle,
        requested: &[PropertyId],
    ) {
        let keyword = |property| match computed.get(property).map(|value| value.value()) {
            Some(PropertyValue::Keyword(symbol)) => Some(*symbol),
            _ => None,
        };
        let rtl = keyword(PropertyId::Direction) == Some(layout_keyword::DIRECTION_RTL);
        let writing_mode = match keyword(PropertyId::WritingMode) {
            Some(layout_keyword::WRITING_MODE_VERTICAL_RL) => WritingMode::VerticalRl,
            Some(layout_keyword::WRITING_MODE_VERTICAL_LR) => WritingMode::VerticalLr,
            _ => WritingMode::HorizontalTb,
        };
        let overflow = requested
            .iter()
            .any(|id| matches!(id, PropertyId::OverflowX | PropertyId::OverflowY))
            .then(|| OverflowAxes::of(Some(computed)));
        let values = requested
            .iter()
            .filter_map(|&property| {
                let physical = property
                    .physical_under(rtl, writing_mode)
                    .unwrap_or(property);
                let value = match physical {
                    PropertyId::OverflowX => PropertyValue::Keyword(overflow?.inline_axis()),
                    PropertyId::OverflowY => PropertyValue::Keyword(overflow?.block_axis()),
                    _ => computed
                        .get(physical)
                        .map(|value| value.value().clone())
                        .or_else(|| style_query_initial(physical))?,
                };
                Some((property, (physical, value)))
            })
            .collect();
        self.styles.insert(
            node,
            StyleContainer {
                name,
                properties: computed.shared_custom_properties(),
                values,
                ranges: FastMap::default(),
                range_context: None,
                font_size_context: None,
                references: FastMap::default(),
            },
        );
    }

    /// Snapshots numeric custom values for the style ranges required by active rules.
    pub fn record_style_ranges(
        &mut self,
        node: NodeId,
        requested: &[InternedSymbol],
        parser: Option<&dyn CustomValueParser>,
    ) {
        let Some(style) = self.styles.get_mut(&node) else {
            return;
        };
        style.ranges = requested
            .iter()
            .map(|&name| {
                let value = style
                    .properties
                    .get(&name)
                    .and_then(|value| substitute(value, &style.properties));
                let range = match value {
                    None | Some(PropertyValue::Initial) => CustomRangeValue::Missing,
                    Some(PropertyValue::RawTokens(tokens))
                        if tokens.eq_ignore_ascii_case("initial") =>
                    {
                        CustomRangeValue::Missing
                    }
                    Some(value) => {
                        let number = match value {
                            PropertyValue::RawTokens(tokens) => {
                                parser.and_then(|parser| parser.parse_style_range(tokens))
                            }
                            value => StyleRangeValue::from_property(value),
                        };
                        number.map_or(CustomRangeValue::Invalid, CustomRangeValue::Numeric)
                    }
                };
                (name, range)
            })
            .collect();
    }

    /// Records custom property tokens lowered in each requested standard property’s grammar.
    pub fn record_style_references(
        &mut self,
        node: NodeId,
        requested: &[(PropertyId, InternedSymbol)],
        parser: Option<&dyn CustomValueParser>,
    ) {
        let Some(style) = self.styles.get_mut(&node) else {
            return;
        };
        style.references = requested
            .iter()
            .filter_map(|&(property, name)| {
                let value = substitute(style.properties.get(&name)?, &style.properties)?;
                if matches!(value, PropertyValue::Initial)
                    || matches!(value, PropertyValue::RawTokens(tokens) if tokens.eq_ignore_ascii_case("initial"))
                {
                    return None;
                }
                let resolved = match value {
                    PropertyValue::RawTokens(tokens) => parser
                        .and_then(|parser| parser.parse_style_property(property, tokens))
                        .map_or(PropertyValue::Unset, |parsed| {
                            substituted_value(property, &parsed, None)
                        }),
                    value => substituted_value(property, value, None),
                };
                Some(((property, name), resolved))
            })
            .collect();
    }

    /// Whether a recorded style query operand on this node needs the font’s zero advance.
    #[must_use]
    pub fn style_ranges_use_ch(&self, node: NodeId) -> bool {
        self.styles.get(&node).is_some_and(|style| {
            style
                .ranges
                .values()
                .any(|range| range.numeric().is_some_and(StyleRangeValue::uses_ch))
                || style.references.values().any(PropertyValue::uses_ch)
        })
    }

    /// Records the selected node’s font, viewport, and writing mode for relative numeric comparisons.
    pub fn set_style_range_context(
        &mut self,
        node: NodeId,
        context: LengthContext,
        vertical: bool,
    ) {
        if let Some(style) = self.styles.get_mut(&node) {
            style.range_context = Some((context, vertical));
        }
    }

    /// Records the parent font metrics and root font basis used by expected font-size values.
    pub fn set_font_size_context(&mut self, node: NodeId, context: LengthContext) {
        if let Some(style) = self.styles.get_mut(&node) {
            style.font_size_context = Some(context);
        }
    }

    pub(super) fn resolve_style_range_length(&self, node: NodeId, length: Length) -> Option<f32> {
        self.query_length_context(node, false, is_container_length(length))
            .and_then(|context| length.resolve_px(&context))
    }

    fn query_length_context(
        &self,
        node: NodeId,
        font_size: bool,
        container_units: bool,
    ) -> Option<LengthContext> {
        let style = self.styles.get(&node)?;
        let (context, vertical) = style.range_context?;
        let mut context = if font_size {
            style.font_size_context?
        } else {
            context
        };
        if container_units {
            context = context.with_query_unit_dimensions(self.unit_dimensions(node), vertical);
        }
        Some(context)
    }

    pub(super) fn resolve_style_range_math(
        &self,
        node: NodeId,
        expr: &LengthExpr,
    ) -> Option<StyleRangeValue> {
        if expr.only_percentages() {
            return expr
                .resolve_with(&|length| match length {
                    Length::Percent(value) => Some(value),
                    _ => None,
                })
                .map(StyleRangeValue::Percent);
        }
        if expr.uses_percent() {
            return None;
        }
        expr.resolve(&self.query_length_context(node, false, true)?)
            .map(StyleRangeValue::Pixels)
    }

    pub(super) fn style_range(
        &self,
        node: NodeId,
        property: InternedSymbol,
    ) -> Option<&CustomRangeValue> {
        self.styles.get(&node)?.ranges.get(&property)
    }

    pub(super) fn test_property(
        &self,
        node: NodeId,
        property: PropertyId,
        expected: Option<&StylePropertyValue>,
    ) -> Option<bool> {
        let style = self.styles.get(&node)?;
        let (physical, actual) = style.values.get(&property)?;
        let expected = match expected {
            Some(value) => {
                Some(value.resolve_with(|name| Some(style.references.get(&(property, name))))?)
            }
            None => None,
        };
        match expected {
            None => Some(actual != &style_query_initial(*physical)?),
            Some(PropertyValue::Initial) => Some(actual == &style_query_initial(*physical)?),
            Some(expected @ (PropertyValue::Length(_) | PropertyValue::Calc(_))) => {
                let expected = self.normalize_query_math(node, property, expected)?;
                if !matches!(actual, PropertyValue::Length(_) | PropertyValue::Calc(_)) {
                    return Some(false);
                }
                Some(self.normalize_query_math(node, property, actual)? == expected)
            }
            Some(PropertyValue::Unset | PropertyValue::Inherit) => None,
            Some(value) => Some(actual == value),
        }
    }

    fn normalize_query_math(
        &self,
        node: NodeId,
        property: PropertyId,
        value: &PropertyValue,
    ) -> Option<PropertyValue> {
        match value {
            PropertyValue::Length(length) => self
                .normalize_query_length(node, property, *length)
                .map(PropertyValue::Length),
            PropertyValue::Calc(expr) => {
                let is_font_size = property == PropertyId::FontSize;
                if !property.supports_style_query_length() {
                    return None;
                }
                let mut context = self.query_length_context(node, is_font_size, true)?;
                if is_font_size {
                    // Font-size math uses the parent's percentage basis and the
                    // same ch fallback as an ordinary font-size expression.
                    context.ch_width = None;
                    return expr
                        .resolve(&context.with_percent_basis(context.font_size))
                        .filter(|value| value.is_finite())
                        .map(|value| PropertyValue::Length(Length::px(value)));
                }
                let value = absolutize_value(PropertyValue::Calc(expr.clone()), &context);
                match &value {
                    PropertyValue::Length(Length::Px(px)) if px.0.is_finite() => Some(value),
                    PropertyValue::Calc(_) if expr.uses_percent() => Some(value),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn normalize_query_length(
        &self,
        node: NodeId,
        property: PropertyId,
        length: Length,
    ) -> Option<Length> {
        if property == PropertyId::FontSize {
            return match length {
                Length::Auto => None,
                Length::Px(_) => Some(length),
                length => {
                    let context =
                        self.query_length_context(node, true, is_container_length(length))?;
                    Some(Length::px(absolute_font_size(
                        Some(length),
                        context,
                        context.ch_width,
                    )))
                }
            };
        }
        match length {
            Length::Px(_) | Length::Percent(_) | Length::Auto => Some(length),
            _ if property.supports_style_query_length() => self
                .resolve_style_range_length(node, length)
                .map(Length::px),
            _ => None,
        }
    }

    /// Sets the list of names recognized on a node for both size and style queries.
    pub fn set_names(&mut self, node: NodeId, names: Arc<[Box<str>]>) {
        self.aliases.insert(node, names);
    }

    fn name_matches(&self, node: NodeId, primary: Option<&str>, requested: Option<&str>) -> bool {
        requested.is_none_or(|name| {
            primary == Some(name)
                || self
                    .aliases
                    .get(&node)
                    .is_some_and(|aliases| aliases.iter().any(|alias| alias.as_ref() == name))
        })
    }

    pub(super) fn find_style(&self, subject: NodeId, name: Option<&str>) -> Option<NodeId> {
        let mut current = subject;
        for _ in 0..self.parents.len() {
            current = *self.parents.get(&current)?;
            if let Some(style) = self.styles.get(&current)
                && self.name_matches(current, style.name.as_deref(), name)
            {
                return Some(current);
            }
        }
        None
    }

    pub(super) fn matches_style(
        &self,
        node: NodeId,
        property: InternedSymbol,
        expected: Option<&PropertyValue>,
    ) -> bool {
        let Some(style) = self.styles.get(&node) else {
            return false;
        };
        let Some(actual) = style
            .properties
            .get(&property)
            .and_then(|value| substitute(value, &style.properties))
        else {
            return false;
        };
        match expected {
            None => {
                !matches!(actual, PropertyValue::Initial | PropertyValue::Unset)
                    && !matches!(actual, PropertyValue::RawTokens(tokens) if tokens.eq_ignore_ascii_case("initial"))
            }
            Some(value) => substitute(value, &style.properties) == Some(actual),
        }
    }

    /// Records a logical parent link for query resolution.
    pub fn link(&mut self, child: NodeId, parent: NodeId) {
        self.parents.insert(child, parent);
    }

    /// Reports whether the snapshot contains any query containers.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.containers.is_empty() && self.styles.is_empty()
    }

    /// Returns the nearest query container size per axis for a retained node, in width, height, inline, block order.
    #[must_use]
    pub fn unit_dimensions(&self, subject: NodeId) -> [Option<f32>; 4] {
        self.unit_dimensions_from(self.parents.get(&subject).copied())
    }

    /// Returns the query unit bases for a child of the given node, including the node itself in the search.
    #[must_use]
    pub fn descendant_unit_dimensions(&self, parent: NodeId) -> [Option<f32>; 4] {
        self.unit_dimensions_from(Some(parent))
    }

    fn unit_dimensions_from(&self, mut ancestor: Option<NodeId>) -> [Option<f32>; 4] {
        let mut dimensions = [None; 4];
        if self.containers.is_empty() {
            return dimensions;
        }
        for _ in 0..=self.parents.len() {
            let Some(node) = ancestor else { break };
            if let Some(container) = self.containers.get(&node) {
                for (base, size) in dimensions.iter_mut().zip(container.dimensions()) {
                    *base = base.or(size);
                }
                if dimensions.iter().all(Option::is_some) {
                    break;
                }
            }
            ancestor = self.parents.get(&node).copied();
        }
        dimensions
    }

    pub(super) fn find(
        &self,
        subject: NodeId,
        name: Option<&str>,
        required: [bool; 4],
    ) -> Option<(NodeId, &QueryContainer)> {
        let mut current = subject;
        for _ in 0..self.parents.len() {
            current = *self.parents.get(&current)?;
            if let Some(container) = self.containers.get(&current)
                && self.name_matches(current, container.name.as_deref(), name)
                && supports_dimensions(container.dimensions(), required)
            {
                return Some((current, container));
            }
        }
        None
    }
}

fn is_container_length(length: Length) -> bool {
    matches!(
        length,
        Length::Cqw(_)
            | Length::Cqh(_)
            | Length::Cqi(_)
            | Length::Cqb(_)
            | Length::Cqmin(_)
            | Length::Cqmax(_)
    )
}

fn style_query_initial(property: PropertyId) -> Option<PropertyValue> {
    let keyword = match property {
        PropertyId::Display => layout_keyword::DISPLAY_BLOCK,
        PropertyId::MaxWidth | PropertyId::MaxHeight => layout_keyword::SIZING_NONE,
        PropertyId::Position => layout_keyword::POSITION_STATIC,
        PropertyId::FlexDirection => layout_keyword::FLEX_ROW,
        PropertyId::FlexWrap => layout_keyword::FLEX_WRAP_NOWRAP,
        PropertyId::AlignContent | PropertyId::JustifyContent => layout_keyword::JUSTIFY_NORMAL,
        PropertyId::AlignItems => layout_keyword::ALIGN_NORMAL,
        PropertyId::JustifyItems => layout_keyword::ALIGN_LEGACY,
        PropertyId::AlignSelf | PropertyId::JustifySelf => layout_keyword::ALIGN_AUTO,
        PropertyId::ZIndex => layout_keyword::Z_INDEX_AUTO,
        PropertyId::BoxSizing => layout_keyword::BOX_SIZING_CONTENT_BOX,
        PropertyId::Direction => layout_keyword::DIRECTION_LTR,
        PropertyId::WritingMode => layout_keyword::WRITING_MODE_HORIZONTAL_TB,
        PropertyId::PointerEvents => layout_keyword::POINTER_EVENTS_AUTO,
        PropertyId::Isolation => layout_keyword::ISOLATION_AUTO,
        PropertyId::MixBlendMode => layout_keyword::MIX_BLEND_MODE_NORMAL,
        PropertyId::OverflowX | PropertyId::OverflowY => layout_keyword::OVERFLOW_VISIBLE,
        PropertyId::OverscrollBehaviorX | PropertyId::OverscrollBehaviorY => {
            InternedSymbol::named("auto")
        }
        PropertyId::TextAlign => layout_keyword::TEXT_ALIGN_START,
        PropertyId::TextOverflow => layout_keyword::TEXT_OVERFLOW_CLIP,
        PropertyId::TextTransform => layout_keyword::TEXT_TRANSFORM_NONE,
        PropertyId::WhiteSpace => layout_keyword::WHITE_SPACE_NORMAL,
        PropertyId::OverflowWrap => layout_keyword::OVERFLOW_WRAP_NORMAL,
        PropertyId::WordBreak => layout_keyword::WORD_BREAK_NORMAL,
        _ => return initial_value_of(property).map(|value| value.value().clone()),
    };
    Some(PropertyValue::Keyword(keyword))
}

pub(super) fn supports_dimensions(dimensions: [Option<f32>; 4], required: [bool; 4]) -> bool {
    dimensions
        .into_iter()
        .zip(required)
        .all(|(size, needed)| !needed || size.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_unit_axes_can_come_from_different_writing_modes() {
        let outer = NodeId::from_raw(1);
        let inner = NodeId::from_raw(2);
        let subject = NodeId::from_raw(3);
        let mut queries = ContainerQueryEnvironment::default();
        queries.insert(
            outer,
            QueryContainer {
                name: None,
                width: Some(240.0),
                height: Some(160.0),
                vertical: false,
            },
        );
        queries.insert(
            inner,
            QueryContainer {
                name: None,
                width: None,
                height: Some(0.0),
                vertical: true,
            },
        );
        queries.link(inner, outer);
        queries.link(subject, inner);

        assert_eq!(
            queries.unit_dimensions(subject),
            [Some(240.0), Some(0.0), Some(0.0), Some(160.0)]
        );
        assert_eq!(queries.unit_dimensions(outer), [None; 4]);
    }
}
