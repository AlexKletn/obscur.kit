#![doc = include_str!("logical_cascade.doc/_module.md")]

use crate::cascade::ResolvedDeclarations;
use crate::computed::ComputedStyle;
use crate::property::{PropertyId, PropertyValue, WritingMode, layout_keyword};

#[doc = include_str!("logical_cascade.doc/resolves_rtl.md")]
pub(super) fn resolves_rtl(
    cascaded: &ResolvedDeclarations,
    parent: Option<&ComputedStyle>,
) -> bool {
    if let Some(PropertyValue::Keyword(symbol)) = cascaded.get(PropertyId::Direction) {
        return *symbol == layout_keyword::DIRECTION_RTL;
    }
    parent
        .and_then(|style| style.get(PropertyId::Direction))
        .and_then(|value| match value.value() {
            PropertyValue::Keyword(symbol) => Some(*symbol),
            _ => None,
        })
        .is_some_and(|symbol| symbol == layout_keyword::DIRECTION_RTL)
}

#[doc = include_str!("logical_cascade.doc/resolves_writing_mode.md")]
pub(super) fn resolves_writing_mode(
    cascaded: &ResolvedDeclarations,
    parent: Option<&ComputedStyle>,
) -> WritingMode {
    let symbol = if let Some(PropertyValue::Keyword(symbol)) = cascaded.get(PropertyId::WritingMode)
    {
        Some(*symbol)
    } else {
        parent
            .and_then(|style| style.get(PropertyId::WritingMode))
            .and_then(|value| match value.value() {
                PropertyValue::Keyword(symbol) => Some(*symbol),
                _ => None,
            })
    };
    match symbol {
        Some(symbol) if symbol == layout_keyword::WRITING_MODE_VERTICAL_RL => {
            WritingMode::VerticalRl
        }
        Some(symbol) if symbol == layout_keyword::WRITING_MODE_VERTICAL_LR => {
            WritingMode::VerticalLr
        }
        _ => WritingMode::HorizontalTb,
    }
}

#[doc = include_str!("logical_cascade.doc/cascaded_for.md")]
pub(super) fn cascaded_for(
    cascaded: &ResolvedDeclarations,
    property: PropertyId,
    rtl: bool,
    wm: WritingMode,
) -> Option<&PropertyValue> {
    let physical = cascaded.get(property);
    // Only a physical box-model property can have a logical counterpart, so
    // this scan is skipped entirely for the other ~80 properties.
    let logical = PropertyId::LOGICAL
        .iter()
        .copied()
        .filter(|candidate| candidate.physical_under(rtl, wm) == Some(property))
        .filter_map(|candidate| Some((candidate, cascaded.get(candidate)?)))
        .max_by_key(|(candidate, _)| cascaded.priority_of(*candidate));
    let Some((candidate, logical_value)) = logical else {
        return physical;
    };
    if physical.is_none() {
        return Some(logical_value);
    }
    match (
        cascaded.priority_of(candidate),
        cascaded.priority_of(property),
    ) {
        (Some(logical_priority), Some(physical_priority))
            if logical_priority > physical_priority =>
        {
            Some(logical_value)
        }
        _ => physical,
    }
}
