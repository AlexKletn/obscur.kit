//! The TSX-like proc-macro DSL and the `#[derive(Contract)]` derive (spec §2.8,
//! §2.13).
//!
//! # `okui!` compiles to ordinary code, not a runtime
//!
//! [`okui!`] is the spec's *primary* authoring surface (the `.okui` file codegen
//! is the fallback). It does **not** hand a document to an interpreter: during
//! expansion the DSL is parsed, lowered to UI IR, and run through `ViewCodegen`
//! — the same lowering the `.okui` build step performs — so what lands in the
//! crate is plain Rust building a `View`. No IR, no `.okui` machinery and no
//! interpretation exist at runtime.
//!
//! Because this is a `proc-macro` crate it can export only macros; the expansion
//! logic lives in the private `dsl` / `derive` modules and is covered by their
//! own unit tests.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod derive;
mod dsl;

use proc_macro::TokenStream;

use obscur_okui_compiler::codegen::{CodegenRoot, ViewCodegen};

use crate::derive::ContractDerive;
use crate::dsl::DslMacro;

/// Builds a `View` from a TSX-like element tree, lowered to ordinary Rust at
/// compile time.
///
/// ```ignore
/// let view = okui! {
///     <container class="row">
///         <Button label="OK" on:click={on_ok}/>
///         "Ready"
///     </container>
/// };
/// ```
///
/// An UPPERCASE-initial tag (`Button`) is a component and resolves to its real
/// `ComponentId`; a lowercase tag (`container`) is an element. See the `dsl`
/// module's docs for the full grammar.
///
/// The expansion is an expression and needs no `use` items — every emitted path
/// is absolute, and written through the `obscur-kit` facade
/// (`::obscur_kit::view::view::…`, `::obscur_kit::identity::…`). So the calling
/// crate needs exactly the one dependency an application already has, and no
/// internal crate: that is [`CodegenRoot::facade`], and the emitted paths used
/// to be `::obscur_view` / `::obscur_identity` — which resolve only inside this
/// workspace, because a `::`-prefixed path is looked up in the crate's *direct*
/// dependencies.
///
/// Malformed input (an unclosed or mismatched tag, a non-expression `{...}`) is
/// reported as a compile error at the offending span.
#[proc_macro]
pub fn okui(input: TokenStream) -> TokenStream {
    // Parse + lower to IR while the macro runs...
    let document = match DslMacro::to_document(input.into()) {
        Ok(document) => document,
        Err(error) => return error.to_compile_error().into(),
    };
    // ...then emit the `View` expression itself, so no IR survives to runtime.
    // Through the facade: an application depends on `obscur-kit` alone, and that
    // is the only root that resolves in its crate root (see `CodegenRoot`).
    let expr = ViewCodegen::document_to_view_expr(&document, &CodegenRoot::facade());
    match expr.parse() {
        Ok(tokens) => tokens,
        // `ViewCodegen` emits Rust we control, so this is unreachable in
        // practice; surface it as a diagnostic rather than panicking blind.
        Err(error) => syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("generated view expression did not parse: {error}"),
        )
        .to_compile_error()
        .into(),
    }
}

/// Derives a typed `ComponentContract` for a props struct (spec §2.13).
///
/// Generates an inherent `contract()` associated function describing the
/// component's props and events, so misuse is caught without hand-registering a
/// surface. Only structs with named fields are supported.
#[proc_macro_derive(Contract, attributes(contract))]
pub fn contract(input: TokenStream) -> TokenStream {
    match ContractDerive::expand(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
