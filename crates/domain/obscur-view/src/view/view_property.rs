//! `ViewProperty`: one property set on a desired view node.

use obscur_identity::InternedSymbol;

use super::ViewValue;

/// A single property set on a node in the desired view tree.
///
/// Data flows *down* into a component as properties (spec §3.1). The property
/// name is an [`InternedSymbol`] rather than an owned string so the desired
/// tree stays keyed on stable interned handles (spec §2.15), and the value is a
/// [`ViewValue`] — a Phase 1 placeholder value (see [`ViewValue`]'s docs).
///
/// Because [`ViewValue`] contains an `f64`, this type derives only `Debug`,
/// `Clone`, and `PartialEq`.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewProperty {
    /// Interned name of the property.
    name: InternedSymbol,
    /// Concrete value assigned to the property.
    value: ViewValue,
}

impl ViewProperty {
    /// Creates a property binding a `name` to a `value`.
    #[must_use]
    pub fn new(name: InternedSymbol, value: ViewValue) -> Self {
        Self { name, value }
    }

    /// Returns the interned property name.
    #[must_use]
    pub const fn name(&self) -> InternedSymbol {
        self.name
    }

    /// Returns the assigned value.
    #[must_use]
    pub const fn value(&self) -> &ViewValue {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_fields() {
        let name = InternedSymbol::from_raw(42);
        let prop = ViewProperty::new(name, ViewValue::Int(9));
        assert_eq!(prop.name(), name);
        assert_eq!(prop.value(), &ViewValue::Int(9));
    }
}
