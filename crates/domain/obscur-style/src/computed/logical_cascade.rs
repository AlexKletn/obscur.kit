#![doc = include_str!("../../docs/api/computed/logical_cascade.doc/_module.md")]

use crate::cascade::ResolvedDeclarations;
use crate::computed::ComputedStyle;
use crate::property::{PropertyId, PropertyValue, WritingMode, layout_keyword};

#[doc = include_str!("../../docs/api/computed/logical_cascade.doc/resolves_rtl.md")]
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

#[doc = include_str!("../../docs/api/computed/logical_cascade.doc/resolves_writing_mode.md")]
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

#[doc = include_str!("../../docs/api/computed/logical_cascade.doc/cascaded_for.md")]
pub(super) fn cascaded_for(
    cascaded: &ResolvedDeclarations,
    property: PropertyId,
    rtl: bool,
    wm: WritingMode,
) -> Option<&PropertyValue> {
    let physical = cascaded.get(property);
    // Only a physical box-model property can have a logical counterpart, so the
    // scan below is skipped entirely for the other ~80 properties. The comment
    // said so long before the guard existed; without it, resolving one node's
    // hundred properties asked `physical_under` two thousand times, most of them
    // to discover that `color` has no logical spelling.
    if !property.is_logical_target() || !cascaded.has_logical() {
        // Two guards, and the second is the one that matters for real
        // stylesheets: a sheet written in `margin-left` rather than
        // `margin-inline-start` has no logical declaration anywhere, and the scan
        // below then walks twenty candidates per box-model property per node to
        // find nothing at all.
        return physical;
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::computed::computed_property_set::COMPUTED_PROPERTIES;

    #[test]
    fn the_two_directions_of_the_logical_mapping_agree() {
        // The guard in `cascaded_for` is a shortcut past `physical_under`, so it
        // is only sound while the two agree. Adding a logical property without
        // teaching `is_logical_target` about its physical side would silently
        // stop that property from cascading, which is a correctness bug wearing
        // an optimisation's clothes — this is what refuses to let that compile
        // quietly.
        for logical in PropertyId::LOGICAL {
            for rtl in [false, true] {
                for wm in [
                    WritingMode::HorizontalTb,
                    WritingMode::VerticalRl,
                    WritingMode::VerticalLr,
                ] {
                    let physical = logical
                        .physical_under(rtl, wm)
                        .expect("every logical property maps to a physical one");
                    assert!(
                        physical.is_logical_target(),
                        "{logical:?} resolves onto {physical:?} under \
                         (rtl={rtl}, {wm:?}), but {physical:?} does not admit it"
                    );
                }
            }
        }
    }

    #[test]
    fn nothing_claims_to_be_a_target_that_no_logical_property_reaches() {
        // The other direction, so the guard cannot quietly widen back into the
        // full scan it exists to avoid.
        let mut reachable: Vec<PropertyId> = PropertyId::LOGICAL
            .iter()
            .flat_map(|logical| {
                [false, true].into_iter().flat_map(move |rtl| {
                    [
                        WritingMode::HorizontalTb,
                        WritingMode::VerticalRl,
                        WritingMode::VerticalLr,
                    ]
                    .into_iter()
                    .filter_map(move |wm| logical.physical_under(rtl, wm))
                })
            })
            .collect();
        reachable.sort_by_key(|property| format!("{property:?}"));
        for physical in &reachable {
            assert!(physical.is_logical_target());
        }
        let claimed = COMPUTED_PROPERTIES
            .iter()
            .filter(|property| property.is_logical_target())
            .count();
        assert_eq!(
            claimed,
            {
                let mut distinct: Vec<PropertyId> = reachable.clone();
                distinct.dedup();
                distinct.len()
            },
            "every property that admits a logical counterpart must be one some \
             logical property actually reaches"
        );
    }
}
