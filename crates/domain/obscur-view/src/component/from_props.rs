//! `FromProps`: turning a node's authored properties into a component's typed props.

use obscur_identity::InternedSymbol;

use crate::component::{FromViewValue, PropsError};
use crate::view::{ViewProperty, ViewValue};

#[doc = include_str!("../../docs/api/component/from_props.doc/FromProps.md")]
pub trait FromProps: Sized {
    #[doc = include_str!("../../docs/api/component/from_props.doc/from_props.md")]
    fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError>;
}

#[doc = include_str!("../../docs/api/component/from_props.doc/prop_value.md")]
#[must_use]
pub fn prop_value(props: &[ViewProperty], name: InternedSymbol) -> Option<&ViewValue> {
    props
        .iter()
        .find(|prop| prop.name() == name)
        .map(ViewProperty::value)
}

#[doc = include_str!("../../docs/api/component/from_props.doc/decode_required.md")]
pub fn decode_required<T: FromViewValue>(
    props: &[ViewProperty],
    name: InternedSymbol,
    label: &'static str,
) -> Result<T, PropsError> {
    match prop_value(props, name) {
        None | Some(ViewValue::Null) => Err(PropsError::missing(name, label)),
        Some(value) => decode_value(value, name, label),
    }
}

#[doc = include_str!("../../docs/api/component/from_props.doc/decode_optional.md")]
pub fn decode_optional<T: FromViewValue>(
    props: &[ViewProperty],
    name: InternedSymbol,
    label: &'static str,
) -> Result<Option<T>, PropsError> {
    match prop_value(props, name) {
        None | Some(ViewValue::Null) => Ok(None),
        Some(value) => decode_value(value, name, label).map(Some),
    }
}

#[doc = include_str!("../../docs/api/component/from_props.doc/decode_value.md")]
pub fn decode_value<T: FromViewValue>(
    value: &ViewValue,
    name: InternedSymbol,
    label: &'static str,
) -> Result<T, PropsError> {
    T::from_view_value(value).ok_or(PropsError::wrong_type(name, label, T::TYPE_NAME))
}

#[cfg(test)]
mod tests {
    use super::{FromProps, decode_optional, decode_required, prop_value};
    use crate::component::PropsError;
    use crate::view::{ViewProperty, ViewValue};
    use obscur_identity::InternedSymbol;

    fn sym(raw: u32) -> InternedSymbol {
        InternedSymbol::from_raw(raw)
    }

    /// `{ #1: Text("hi"), #2: Bool(true), #3: Null }`.
    fn props() -> Vec<ViewProperty> {
        vec![
            ViewProperty::new(sym(1), ViewValue::Text("hi".into())),
            ViewProperty::new(sym(2), ViewValue::Bool(true)),
            ViewProperty::new(sym(3), ViewValue::Null),
        ]
    }

    #[test]
    fn prop_value_finds_by_symbol_and_misses_cleanly() {
        assert_eq!(prop_value(&props(), sym(2)), Some(&ViewValue::Bool(true)));
        assert_eq!(prop_value(&props(), sym(99)), None);
    }

    #[test]
    fn a_required_prop_decodes_or_reports_missing() {
        assert_eq!(decode_required::<bool>(&props(), sym(2), ""), Ok(true));
        assert_eq!(
            decode_required::<bool>(&props(), sym(99), ""),
            Err(PropsError::missing(sym(99), ""))
        );
    }

    #[test]
    fn null_counts_as_absent_for_required_and_optional_alike() {
        // `Null` IS absence of a value, so it must not decode as "present".
        assert_eq!(
            decode_required::<bool>(&props(), sym(3), ""),
            Err(PropsError::missing(sym(3), ""))
        );
        assert_eq!(decode_optional::<bool>(&props(), sym(3), ""), Ok(None));
    }

    #[test]
    fn an_optional_prop_is_none_when_absent_but_still_errors_on_a_bad_type() {
        assert_eq!(decode_optional::<String>(&props(), sym(99), ""), Ok(None));
        // The important half: a wrongly-typed value must NOT degrade to `None`,
        // or an authored mistake would look like "not supplied".
        assert_eq!(
            decode_optional::<bool>(&props(), sym(1), ""),
            Err(PropsError::wrong_type(sym(1), "", "bool"))
        );
    }

    #[test]
    fn a_wrong_type_names_the_wanted_type() {
        assert_eq!(
            decode_required::<bool>(&props(), sym(1), ""),
            Err(PropsError::wrong_type(sym(1), "", "bool"))
        );
    }

    /// A hand-written impl, standing in for what `#[derive(Contract)]` emits.
    #[derive(Debug, PartialEq)]
    struct Probe {
        label: String,
        enabled: Option<bool>,
    }

    impl FromProps for Probe {
        fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
            Ok(Self {
                label: decode_required(props, sym(1), "label")?,
                enabled: decode_optional(props, sym(2), "enabled")?,
            })
        }
    }

    #[test]
    fn a_props_struct_round_trips_through_the_trait() {
        assert_eq!(
            Probe::from_props(&props()),
            Ok(Probe {
                label: "hi".to_owned(),
                enabled: Some(true),
            })
        );
        // Missing the required field fails the whole decode.
        assert_eq!(
            Probe::from_props(&[]),
            Err(PropsError::missing(sym(1), "label"))
        );
    }
}
