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
//! - [`ContractValidator`] checks a component instance against a
//!   [`ComponentContract`] — required props and allowed children (§2.13).

mod builtin_contracts;
pub use builtin_contracts::{builtin_contracts, widget_contracts};
mod capability_validator;
pub use capability_validator::CapabilityValidator;
mod contract_validator;
pub use contract_validator::{ComponentContract, ContractValidator};
mod expression_validator;
pub use expression_validator::{ExpressionValidator, ValidationDiagnostic};
mod surface_bridge;
