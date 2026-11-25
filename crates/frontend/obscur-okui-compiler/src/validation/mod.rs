#![doc = include_str!("../../docs/api/validation/mod.doc/_module.md")]

mod builtin_contracts;
pub use builtin_contracts::{
    builtin_contracts, default_contracts, mounted_element_contracts, widget_contracts,
};
mod capability_validator;
pub use capability_validator::CapabilityValidator;
mod child_policy;
pub use child_policy::ChildPolicy;
mod command_surface;
pub use command_surface::CommandSurface;
mod command_validator;
pub use command_validator::CommandValidator;
pub mod collection_rule;
pub use collection_rule::{
    DUPLICATE_KEY, KEY_REQUIRES_VALUE, check_collection, duplicate_collection_slots,
};
mod collection_validator;
pub use collection_validator::CollectionValidator;
mod conditional_rule;
pub use conditional_rule::{check_condition_attribute, check_conditional_branch};
pub mod contract_rule;
pub use contract_rule::{check_contract, is_reserved_attribute};
mod contract_validator;
pub use contract_validator::ContractValidator;
mod expression_validator;
pub use expression_validator::{
    ExpressionValidator, FORBIDDEN_FREE_CALL, FORBIDDEN_MESSAGE, check_expression,
    check_expression_in,
};
pub mod outlet_rule;
pub use outlet_rule::{
    DYNAMIC_NAME_MESSAGE, MISSING_NAME_MESSAGE, OUTLET_CHILDREN_MESSAGE, check_outlet,
    extra_attribute_message, is_outlet_tag,
};
mod outlet_validator;
pub use outlet_validator::OutletValidator;
mod slot_rule;
pub use slot_rule::SlotRule;
mod surface_bridge;

mod validation_diagnostic;
pub use validation_diagnostic::ValidationDiagnostic;
mod outlet_name;
pub use outlet_name::OutletName;
mod outlet_use;
pub use outlet_use::OutletUse;

mod member_arity;
pub use member_arity::MemberArity;

mod attribute_use;
pub use attribute_use::AttributeUse;

mod slot_use;
pub use slot_use::SlotUse;

mod member_use;
pub use member_use::MemberUse;

mod element_use;
pub use element_use::ElementUse;

mod violation_anchor;
pub use violation_anchor::ViolationAnchor;

mod contract_violation;
pub use contract_violation::ContractViolation;

mod component_contract;
pub use component_contract::ComponentContract;
