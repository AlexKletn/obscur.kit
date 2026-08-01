#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod derive;
mod dsl;

use proc_macro::TokenStream;

use obscur_okui_compiler::codegen::{CodegenRoot, ViewCodegen};

use crate::derive::ContractDerive;
use crate::dsl::DslMacro;

#[doc = include_str!("lib.doc/okui.md")]
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

#[doc = include_str!("lib.doc/contract.md")]
#[proc_macro_derive(Contract, attributes(contract))]
pub fn contract(input: TokenStream) -> TokenStream {
    match ContractDerive::expand(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
