#![doc = include_str!("../../docs/api/select/select_props.doc/_module.md")]

use obscur_identity::InternedSymbol;
use obscur_view::component::{ComponentContract, FromProps, PropsError, prop_value};
use obscur_view::contract::PropContract;
use obscur_view::view::{ViewProperty, ViewValue};

use super::Select;

#[doc = include_str!("../../docs/api/select/select_props.doc/SelectProps.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectProps {
    /// The option labels, in the order they are offered.
    pub options: Vec<Box<str>>,
    /// The index of the picked option, or `None` when nothing is picked — in which
    /// case the header shows the first option, as a bare `<select>` does.
    pub selected: Option<usize>,
    /// Whether the dropdown popup is down. The `.select-list` of rows is rendered
    /// exactly when this is `true`, **including on a disabled control**: see
    /// [`Select::contents`](super::Select).
    pub open: bool,
}

impl SelectProps {
    #[doc = include_str!("../../docs/api/select/select_props.doc/OPTIONS.md")]
    pub const OPTIONS: InternedSymbol = InternedSymbol::from_raw(0xEF28_6CA5);
    #[doc = include_str!("../../docs/api/select/select_props.doc/SELECTED.md")]
    pub const SELECTED: InternedSymbol = InternedSymbol::from_raw(0xCE0A_1774);
    #[doc = include_str!("../../docs/api/select/select_props.doc/OPEN.md")]
    pub const OPEN: InternedSymbol = InternedSymbol::from_raw(0xD35E_C4C9);

    #[doc = include_str!("../../docs/api/select/select_props.doc/AUTHORED_SEPARATOR.md")]
    pub const AUTHORED_SEPARATOR: char = ',';

    /// Symbol of the component's name, `"Select"` — the FNV-1a-32 hash
    /// `#[derive(Contract)]` puts on `ComponentContract::with_name`.
    const NAME: InternedSymbol = InternedSymbol::from_raw(0x3E89_2B4D);
    /// Opaque type identity of the `options` prop: the FNV-1a-32 hash of the
    /// normalised Rust type spelling `Option < String >` — the type of the *authored*
    /// value, which is a delimited string, not the `Vec<Box<str>>` this struct
    /// decodes it into. It names no view property and is compared, never interpreted.
    const OPTIONS_TYPE: InternedSymbol = InternedSymbol::from_raw(0x4F1F_A46F);
    /// Opaque type identity of the `selected` prop: the FNV-1a-32 hash of the
    /// normalised Rust type spelling `Option < usize >`, which is what
    /// `#[derive(Contract)]` records for an `Option<usize>` field.
    const SELECTED_TYPE: InternedSymbol = InternedSymbol::from_raw(0x3AC8_8BA4);
    /// Opaque type identity of the `open` prop: the FNV-1a-32 hash of the Rust type
    /// spelling `bool`, which is what `#[derive(Contract)]` records for a plain `bool`
    /// field carrying `#[contract(default)]`.
    const OPEN_TYPE: InternedSymbol = InternedSymbol::from_raw(0xC894_953D);

    #[doc = include_str!("../../docs/api/select/select_props.doc/contract.md")]
    #[must_use]
    pub fn contract() -> ComponentContract {
        ComponentContract::new(Select::COMPONENT_ID)
            .with_name(Self::NAME)
            .with_prop(PropContract::new(Self::OPTIONS, Self::OPTIONS_TYPE).with_default())
            .with_prop(PropContract::new(Self::SELECTED, Self::SELECTED_TYPE).with_default())
            .with_prop(PropContract::new(Self::OPEN, Self::OPEN_TYPE).with_default())
    }
}

impl FromProps for SelectProps {
    #[doc = include_str!("../../docs/api/select/select_props.doc/from_props.md")]
    fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
        let options = match prop_value(props, Select::PROP_OPTIONS) {
            Some(ViewValue::Text(joined)) => joined
                .split(Select::OPTION_SEPARATOR)
                .map(Into::into)
                .collect(),
            // No widget prop: this node came from markup, where the list is the
            // readable comma-separated spelling and an empty one names no options
            // at all.
            _ => match prop_value(props, Self::OPTIONS) {
                Some(ViewValue::Text(authored)) => authored
                    .split(Self::AUTHORED_SEPARATOR)
                    .map(str::trim)
                    .filter(|label| !label.is_empty())
                    .map(Into::into)
                    .collect(),
                _ => Vec::new(),
            },
        };
        let selected = match prop_value(props, Select::PROP_SELECTED) {
            Some(&ViewValue::Int(index)) => usize::try_from(index).ok(),
            _ => match prop_value(props, Self::SELECTED) {
                Some(&ViewValue::Int(index)) => usize::try_from(index).ok(),
                _ => None,
            },
        };
        let open = match prop_value(props, Select::PROP_OPEN) {
            Some(&ViewValue::Bool(flag)) => flag,
            // Markup's spelling is a bare `<select open/>`, which lowers to
            // `ViewValue::Null`: presence is the statement, the way it is for every
            // structural state attribute.
            _ => match prop_value(props, Self::OPEN) {
                Some(&ViewValue::Bool(flag)) => flag,
                Some(ViewValue::Null) => true,
                _ => false,
            },
        };
        Ok(Self {
            options,
            selected,
            open,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SelectProps;
    use crate::select::Select;
    use obscur_identity::InternedSymbol;
    use obscur_view::component::FromProps;
    use obscur_view::view::{ViewNode, ViewProperty, ViewValue};

    fn authored(options: &str) -> ViewProperty {
        ViewProperty::new(SelectProps::OPTIONS, ViewValue::from(options))
    }

    fn decoded(props: &[ViewProperty]) -> SelectProps {
        SelectProps::from_props(props).expect("a select decodes without a required prop")
    }

    #[test]
    fn a_bare_select_decodes_to_an_empty_closed_control() {
        // Every prop is defaulted: a `<select/>` is a legal, empty control, so
        // nothing here may report.
        assert_eq!(
            decoded(&[]),
            SelectProps {
                options: Vec::new(),
                selected: None,
                open: false,
            }
        );
    }

    #[test]
    fn an_authored_list_is_comma_separated_and_trimmed() {
        assert_eq!(
            decoded(&[authored("Alpha, Beta ,Gamma")]).options,
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()]
        );
        // An empty authored list names no options, rather than one empty one.
        assert_eq!(decoded(&[authored("")]).options, Vec::<Box<str>>::new());
        assert_eq!(decoded(&[authored(" , ")]).options, Vec::<Box<str>>::new());
    }

    #[test]
    fn the_widgets_own_list_round_trips_a_label_containing_a_comma() {
        // THE reason there are two spellings. The authored list is comma-separated
        // because a human writes it; a Rust caller's label may contain a comma, and
        // splitting it would silently turn one option into two the day the built-in
        // was registered.
        let built = Select::new(InternedSymbol::from_raw(0x9ABC)).with_options(["a,b", "c"]);
        let view = built.view();
        let ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
            panic!("expected element root");
        };
        assert_eq!(decoded(props).options, vec!["a,b".into(), "c".into()]);
    }

    #[test]
    fn an_empty_label_survives_but_an_empty_control_stays_empty() {
        // Joining `[]` and joining `[""]` both give `""`, so the widget emits the
        // prop only when it has options — and these two must not collapse into one.
        let command = InternedSymbol::from_raw(0x9ABC);
        for (built, expected) in [
            (Select::new(command), Vec::<Box<str>>::new()),
            (Select::new(command).option(""), vec!["".into()]),
        ] {
            let view = built.view();
            let ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
                panic!("expected element root");
            };
            assert_eq!(decoded(props).options, expected, "{built:?}");
        }
    }

    #[test]
    fn a_valueless_open_attribute_opens_the_popup() {
        // `<select open/>` lowers to `ViewValue::Null`; presence is the statement.
        assert!(
            decoded(&[ViewProperty::new(SelectProps::OPEN, ViewValue::Null)]).open,
            "a bare `open` attribute must open the popup"
        );
        assert!(
            !decoded(&[ViewProperty::new(SelectProps::OPEN, ViewValue::Bool(false))]).open,
            "an explicit `open={{false}}` stays closed"
        );
        assert!(!decoded(&[]).open, "an absent `open` stays closed");
    }

    #[test]
    fn the_widgets_own_props_win_over_the_authored_ones() {
        // The Rust-API path. `Select::view` states all three exactly, and reading
        // them first is what makes mounting idempotent on a node the builder made.
        let props = [
            ViewProperty::new(Select::PROP_OPTIONS, ViewValue::from("One\u{1f}Two")),
            ViewProperty::new(Select::PROP_SELECTED, ViewValue::Int(1)),
            ViewProperty::new(Select::PROP_OPEN, ViewValue::Bool(true)),
            authored("Ignored"),
            ViewProperty::new(SelectProps::SELECTED, ViewValue::Int(0)),
            ViewProperty::new(SelectProps::OPEN, ViewValue::Bool(false)),
        ];
        assert_eq!(
            decoded(&props),
            SelectProps {
                options: vec!["One".into(), "Two".into()],
                selected: Some(1),
                open: true,
            }
        );
    }

    #[test]
    fn the_widgets_no_selection_sentinel_is_not_an_index() {
        // `Select::view` spells "nothing picked" as `-1`, which is not a `usize`.
        let props = [ViewProperty::new(Select::PROP_SELECTED, ViewValue::Int(-1))];
        assert_eq!(decoded(&props).selected, None);
    }

    #[test]
    fn decodes_the_node_the_rust_builder_produces() {
        // The idempotence precondition: everything this decoder needs is on the
        // node `Select::view` builds, so mounting the component on one changes
        // nothing.
        let mut built = Select::new(InternedSymbol::from_raw(0x9ABC)).with_options(["One", "Two"]);
        assert!(built.select(1), "the second option must be selectable");
        built.open();
        let view = built.view();
        let ViewNode::Element { props, .. } = &view.root().nodes()[0] else {
            panic!("expected element root");
        };
        assert_eq!(
            decoded(props),
            SelectProps {
                options: vec!["One".into(), "Two".into()],
                selected: Some(1),
                open: true,
            }
        );
    }

    #[test]
    fn the_contract_is_the_id_the_tag_and_the_builder_share() {
        let contract = SelectProps::contract();
        assert_eq!(contract.id, Select::COMPONENT_ID);
        assert_eq!(contract.props.len(), 3);
        assert!(
            contract.props.iter().all(|prop| prop.has_default),
            "a `<select/>` with nothing on it is a legal, empty control"
        );
        assert_eq!(
            contract
                .props
                .iter()
                .map(|prop| prop.name)
                .collect::<Vec<_>>(),
            vec![
                SelectProps::OPTIONS,
                SelectProps::SELECTED,
                SelectProps::OPEN
            ]
        );
        // A select builds its own header, arrow and rows, so authored children have
        // nowhere to go — the default, restated as an assertion so a change is
        // deliberate.
        assert_eq!(
            contract.allowed_children,
            obscur_view::contract::ChildContract::None
        );
    }
}
