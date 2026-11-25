use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::LitStr;

use super::{component_id, slot_cardinality::SlotCardinality};

/// One `#[contract(slot(name = "…", cardinality = …, child = "…"))]` declaration
/// (spec §2.12).
#[derive(Debug)]
pub(super) struct SlotModel {
    /// The slot's name — the value a child's `slot="…"` must match.
    name: String,
    /// How many children the slot accepts; `many` when unstated, the only
    /// default that adds no obligation the author did not ask for.
    cardinality: SlotCardinality,
    /// The child tags the slot admits; empty means any.
    children: Vec<String>,
}

impl SlotModel {
    /// Parses one `slot(...)` list from a struct-level `#[contract(...)]`.
    pub(super) fn parse(meta: &syn::meta::ParseNestedMeta<'_>) -> Result<Self, syn::Error> {
        let mut name: Option<String> = None;
        let mut cardinality = SlotCardinality::Many;
        let mut children: Vec<String> = Vec::new();
        meta.parse_nested_meta(|inner| {
            if inner.path.is_ident("name") {
                let spelled = inner.value()?.parse::<LitStr>()?.value();
                if spelled.is_empty() {
                    return Err(inner.error(
                        "the default slot is spelled `slot(default)`, not `name = \"\"` \
                         (spec §2.12): it is declared under a reserved symbol precisely \
                         so that no author-written name can ever collide with it",
                    ));
                }
                name = Some(spelled);
            } else if inner.path.is_ident("default") {
                // §2.12's default slot: the one an outlet naming no slot fills,
                // and the only one a text run can ever reach.
                name = Some(String::new());
            } else if inner.path.is_ident("cardinality") {
                let word: syn::Ident = inner.value()?.parse()?;
                cardinality = match word.to_string().as_str() {
                    "one" => SlotCardinality::One,
                    "optional" => SlotCardinality::Optional,
                    "many" => SlotCardinality::Many,
                    other => {
                        return Err(inner.error(format!(
                            "unknown `cardinality` value `{other}` (expected `one`, `optional` or `many`)"
                        )));
                    }
                };
            } else if inner.path.is_ident("child") {
                children.push(inner.value()?.parse::<LitStr>()?.value());
            } else {
                return Err(inner.error(
                    "unknown `slot` key (expected `name`, `default`, `cardinality` or `child`)",
                ));
            }
            Ok(())
        })?;
        let Some(name) = name else {
            return Err(meta.error("a `slot(...)` must give a `name = \"…\"` or be `default`"));
        };
        Ok(Self {
            name,
            cardinality,
            children,
        })
    }

    #[doc = include_str!("../../../docs/api/derive/contract_derive.doc/contract_tokens.md")]
    pub(super) fn contract_tokens(&self) -> TokenStream {
        // Not a hash computed here. Every other symbol this derive emits is a
        // literal because the macro can reach the same function the runtime
        // uses; for a slot name the runtime function is `const`, so emitting the
        // call itself is strictly better — there is no second implementation of
        // the rule to drift, and the default slot's reserved symbol is spelled
        // in exactly one place.
        let name_lit = LitStr::new(&self.name, Span::call_site());
        let cardinality = self.cardinality.variant();
        let children = if self.children.is_empty() {
            quote!(::obscur_kit::view::contract::ChildContract::Any)
        } else {
            let ids = self.children.iter().map(|tag| {
                let id = Literal::u64_unsuffixed(component_id(tag));
                quote!(::obscur_kit::identity::ComponentId::from_raw(#id))
            });
            quote!(::obscur_kit::view::contract::ChildContract::Only(
                ::std::vec![#(#ids),*]
            ))
        };
        quote! {
            .with_slot(::obscur_kit::view::contract::SlotContract::new(
                ::obscur_kit::view::view::slot_symbol(#name_lit),
                ::obscur_kit::view::contract::SlotCardinality::#cardinality,
                #children,
            ))
        }
    }

    /// Lowers the slot into its `.with_slot(...)` entry on the **string**
    /// surface, which is the one the `.okui` compiler and the `okui!` macro
    /// actually check a document against.
    pub(super) fn surface_tokens(&self) -> TokenStream {
        // The string surface is what the two front ends check authored markup
        // against, and neither has a spelling for the default slot — content
        // reaches it by naming nothing. Emitting it here would give
        // `check_cardinality` a slot no `slot="…"` can ever satisfy, so a
        // `slot(default, cardinality = one)` would report itself missing on
        // every use.
        if self.name.is_empty() {
            return TokenStream::new();
        }
        let name = LitStr::new(&self.name, Span::call_site());
        let cardinality = self.cardinality.variant();
        let children = self.children.iter().map(|tag| {
            let tag = LitStr::new(tag, Span::call_site());
            quote!(.allowing_child(#tag))
        });
        quote! {
            .with_slot(
                ::obscur_kit::view::contract::SlotSurface::new(
                    #name,
                    ::obscur_kit::view::contract::SlotCardinality::#cardinality,
                )
                #(#children)*
            )
        }
    }
}
