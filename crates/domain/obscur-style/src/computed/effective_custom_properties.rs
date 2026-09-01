#![doc = include_str!("effective_custom_properties.doc/_module.md")]

use crate::computed::ComputedStyle;
use crate::property::PropertyValue;
use obscur_identity::{FastMap, FastSet, InternedSymbol};
use std::collections::{HashMap, HashSet};

#[doc = include_str!("effective_custom_properties.doc/effective_custom_properties.md")]
pub(super) fn effective_custom_properties(
    custom: &FastMap<InternedSymbol, PropertyValue>,
    parent: Option<&ComputedStyle>,
) -> FastMap<InternedSymbol, PropertyValue> {
    let mut effective: FastMap<InternedSymbol, PropertyValue> = HashMap::default();
    if let Some(parent) = parent {
        for (name, value) in parent.custom_properties() {
            effective.insert(*name, value.clone());
        }
    }
    for (name, value) in custom {
        effective.insert(*name, value.clone());
    }
    effective
}

#[doc = include_str!("effective_custom_properties.doc/substitute.md")]
pub(super) fn substitute(
    value: &PropertyValue,
    custom: &FastMap<InternedSymbol, PropertyValue>,
) -> PropertyValue {
    let mut current = value.clone();
    let mut visited: FastSet<InternedSymbol> = HashSet::default();
    loop {
        match current {
            PropertyValue::CustomRef(name) => {
                if !visited.insert(name) {
                    // Cycle: treat as unset.
                    return PropertyValue::Unset;
                }
                match custom.get(&name) {
                    Some(next) => current = next.clone(),
                    // Missing custom property: treat as unset.
                    None => return PropertyValue::Unset,
                }
            }
            other => return other,
        }
    }
}
