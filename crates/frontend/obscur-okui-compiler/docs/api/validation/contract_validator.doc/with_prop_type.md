Records the declared type category of prop `name` (fluent; spec §2.8), so
[`validate_element`](ContractValidator::validate_element) can reject a
literal of the wrong kind. Bridged from a
[`ComponentSurface`](obscur_view::contract::ComponentSurface); a prop with
no recorded type, or one recorded as [`PropTypeSurface::Other`], is not
type-checked.
