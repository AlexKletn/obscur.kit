#![doc = include_str!("../docs/api/lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod derive;
mod dsl;

#[doc = include_str!("../docs/api/lib.doc/okui.md")]
#[proc_macro]
pub fn okui(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    dsl::DslMacro::expand(input.into()).into()
}

#[doc = include_str!("../docs/api/lib.doc/contract.md")]
#[proc_macro_derive(Contract, attributes(contract))]
pub fn contract(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match derive::ContractDerive::expand(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
