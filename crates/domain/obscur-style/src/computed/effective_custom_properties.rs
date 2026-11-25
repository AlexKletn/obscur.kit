#![doc = include_str!("../../docs/api/computed/effective_custom_properties.doc/_module.md")]

use crate::computed::ComputedStyle;
use crate::property::PropertyValue;
use obscur_identity::{FastMap, FastSet, InternedSymbol};
use std::collections::{HashMap, HashSet};

#[doc = include_str!("../../docs/api/computed/effective_custom_properties.doc/effective_custom_properties.md")]
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

#[doc = include_str!("../../docs/api/computed/effective_custom_properties.doc/substitute.md")]
pub(crate) fn substitute<'a>(
    value: &'a PropertyValue,
    custom: &'a FastMap<InternedSymbol, PropertyValue>,
) -> Option<&'a PropertyValue> {
    // Borrowed, never cloned: every value on the chain is owned either by the
    // declaration or by the custom-property map, and both outlive the answer.
    // Cloning each step used to be free enough to ignore, because a custom
    // property came out of the compiler as a lowered value — a colour, a
    // length, a keyword, none of which own heap. A custom property now keeps
    // the TEXT it was declared with, so that same clone became one heap string
    // per `var()` reference, per node, per restyle, made only to be read once
    // and dropped.
    let mut current = value;
    let mut visited: FastSet<InternedSymbol> = HashSet::default();
    loop {
        match current {
            PropertyValue::CustomRef(name) => {
                if !visited.insert(*name) {
                    // Cycle: treat as unset.
                    return None;
                }
                match custom.get(name) {
                    Some(next) => current = next,
                    // Missing custom property: treat as unset.
                    None => return None,
                }
            }
            other => return Some(other),
        }
    }
}
