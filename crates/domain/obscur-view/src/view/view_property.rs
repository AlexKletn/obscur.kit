//! `ViewProperty`: one property set on a desired view node.

use obscur_identity::InternedSymbol;

use super::{ViewValue, virtual_row_builder::VirtualRowBuilder};

#[doc = include_str!("../../docs/api/view/view_property.doc/ViewProperty.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ViewProperty {
    /// Interned name of the property.
    name: InternedSymbol,
    /// Concrete value assigned to the property.
    value: ViewValue,
    virtual_rows: Option<VirtualRowBuilder>,
}

impl ViewProperty {
    /// Creates a property binding a `name` to a `value`.
    #[must_use]
    pub fn new(name: InternedSymbol, value: ViewValue) -> Self {
        Self {
            name,
            value,
            virtual_rows: None,
        }
    }

    pub(super) fn with_virtual_rows(mut self, rows: VirtualRowBuilder) -> Self {
        self.virtual_rows = Some(rows);
        self
    }

    pub(super) fn virtual_rows(&self) -> Option<&VirtualRowBuilder> {
        self.virtual_rows.as_ref()
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
