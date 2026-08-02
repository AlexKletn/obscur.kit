#![doc = include_str!("mod.doc/_module.md")]

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
pub use collection_rule::check_collection;
mod collection_validator;
pub use collection_validator::CollectionValidator;
pub mod contract_rule;
pub use contract_rule::{
    AttributeUse, ContractViolation, ElementUse, MemberArity, MemberUse, SlotUse, ViolationAnchor,
    check_contract, is_reserved_attribute,
};
mod contract_validator;
pub use contract_validator::{ComponentContract, ContractValidator};
mod expression_validator;
pub use expression_validator::{
    ExpressionValidator, FORBIDDEN_FREE_CALL, FORBIDDEN_MESSAGE, SERVICE_ACCESS_MESSAGE,
    ValidationDiagnostic, check_expression, check_expression_in,
};
pub mod outlet_rule;
pub use outlet_rule::{
    DYNAMIC_NAME_MESSAGE, MISSING_NAME_MESSAGE, OUTLET_CHILDREN_MESSAGE, OutletName, OutletUse,
    check_outlet, extra_attribute_message, is_outlet_tag,
};
mod outlet_validator;
pub use outlet_validator::OutletValidator;
mod slot_rule;
pub use slot_rule::SlotRule;
mod surface_bridge;
