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
//! - [`CommandValidator`] checks that every `on:<event>` handler reference names a
//!   command somebody claims (§2.13's "string method names"), against the
//!   [`CommandSurface`] the caller declared. A handler is matched by a hash of its
//!   name, so an unclaimed one used to be a silent no-op — a button that does
//!   nothing, with no diagnostic anywhere. Only the *file* front end can run this:
//!   see [`CommandSurface`] for why the `okui!` macro cannot, and must not guess.
//! - [`CollectionValidator`] applies the §2.11 collection rule
//!   ([`check_collection`], shared with the macro front end): both `<For>` and
//!   `<VirtualFor>` are honoured — the latter lowers to a virtual-collection
//!   marker the runtime realizes into a viewport-bounded window (§4.9) — so the
//!   rule accepts them, and the walk is the seam a future collection constraint
//!   would live in. See [`collection_rule`]'s docs.
//! - [`ContractValidator`] checks a component instance against a
//!   [`ComponentContract`] — required props, allowed children (a three-state
//!   [`ChildPolicy`], so a contract can say *no* children and not only *which*)
//!   and the §2.12 slots — on the **file** front end.
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
