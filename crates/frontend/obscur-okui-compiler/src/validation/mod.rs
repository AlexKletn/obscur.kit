//! Static validation of a resolved `.okui` document (spec §2.10, §2.13).
//!
//! This module hosts the semantic checks that run after name resolution and
//! typing. Every check reports the shared [`ValidationDiagnostic`] — a message
//! plus a source byte offset — so failures compose uniformly:
//!
//! - [`ExpressionValidator`] enforces the pure-expression subset (§2.10): no
//!   assignment, mutation, I/O, async or forbidden keywords in an `{expr}`.
//! - [`CapabilityValidator`] checks that each element's declared required
//!   capability is granted (§2.13).
//! - [`CollectionValidator`] refuses a §2.11 collection construct whose contract
//!   this pipeline does not keep — today `<VirtualFor>`, whose §4.9 windowing
//!   promise nothing honours. The rule is [`check_collection`], shared with the
//!   macro front end for the reason [`collection_rule`]'s docs give.
//! - [`ContractValidator`] checks a component instance against a
//!   [`ComponentContract`] — required props, allowed children and the §2.12
//!   slots — on the **file** front end.
//! - [`check_contract`] is that same rule, stated over plain data, so the
//!   `okui!` macro reaches the same verdict from `syn` tokens; see the
//!   [`contract_rule`] module docs for why the rule is not written against
//!   either front end's input.
//! - [`OutletValidator`] checks an authored `<slot name="…"/>` — the §2.12 outlet
//!   — against [`check_outlet`], which both front ends share. `check_contract`
//!   polices the *assignment* side of the slot join (`<h1 slot="header"/>`); this
//!   polices the *declaration* side, which has a spelling of its own and a
//!   surface of exactly one attribute.

mod builtin_contracts;
pub use builtin_contracts::{builtin_contracts, widget_contracts};
mod capability_validator;
pub use capability_validator::CapabilityValidator;
pub mod collection_rule;
pub use collection_rule::{VIRTUAL_FOR_MESSAGE, check_collection};
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
    ExpressionValidator, FORBIDDEN_MESSAGE, ValidationDiagnostic, check_expression,
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
