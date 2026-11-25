use proc_macro2::{Literal, Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{LitStr, Type, spanned::Spanned};

use super::{event_name_symbol, field_attrs::FieldAttrs, fnv32, name_symbol};

/// The parsed model of a single struct field for contract generation.
#[derive(Debug)]
pub(super) struct FieldModel {
    /// The field's actual Rust identifier, used to initialise it in the
    /// generated `FromProps` impl (the authored `name` may be overridden and so
    /// cannot stand in for it).
    ident: syn::Ident,
    /// The name string used for hashing (field ident or `name = "..."` override).
    name: String,
    /// The normalized textual type string used for the payload/value-type hash.
    type_string: String,
    /// Whether the field is an `Option<...>` (implies optional + default).
    is_option: bool,
    /// Field-level attribute flags.
    attrs: FieldAttrs,
}

impl FieldModel {
    /// Parses a named struct `field` into a [`FieldModel`].
    pub(super) fn parse(field: &syn::Field) -> Result<Self, syn::Error> {
        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new(field.span(), "expected a named contract field"))?;
        let attrs = FieldAttrs::parse(field)?;
        let name = attrs.name.clone().unwrap_or_else(|| ident.to_string());
        Ok(Self {
            ident: ident.clone(),
            name,
            type_string: normalized_type_string(&field.ty),
            is_option: type_is_option(&field.ty),
            attrs,
        })
    }

    #[doc = include_str!("../../../docs/api/derive/contract_derive.doc/from_props_tokens.md")]
    pub(super) fn from_props_tokens(&self) -> TokenStream {
        let ident = &self.ident;

        if self.attrs.skip || self.attrs.event {
            return quote! { #ident: ::core::default::Default::default(), };
        }

        let name_sym = Literal::u32_unsuffixed(name_symbol(&self.name));
        let symbol = quote! { ::obscur_kit::identity::InternedSymbol::from_raw(#name_sym) };
        // The name in words, beside the symbol that replaced it. This is the
        // only point in the whole path where the property still HAS a name: the
        // symbol is an FNV hash with no reverse table, so a decode failure
        // reported downstream could otherwise say only `#2147483911` — which is
        // what made a hyphenated attribute cost an afternoon (audit 1-B).
        let label = Literal::string(&self.name);

        if self.is_option {
            return quote! {
                #ident: ::obscur_kit::view::component::decode_optional(props, #symbol, #label)?,
            };
        }
        if self.attrs.default {
            return quote! {
                #ident: match ::obscur_kit::view::component::prop_value(props, #symbol) {
                    ::core::option::Option::None
                    | ::core::option::Option::Some(::obscur_kit::view::view::ViewValue::Null) => {
                        ::core::default::Default::default()
                    }
                    ::core::option::Option::Some(value) => {
                        ::obscur_kit::view::component::decode_value(value, #symbol, #label)?
                    }
                },
            };
        }
        quote! {
            #ident: ::obscur_kit::view::component::decode_required(props, #symbol, #label)?,
        }
    }

    /// Lowers the field into its typed `.with_prop(...)` / `.with_event(...)`
    /// builder call, or `None` when the field is skipped.
    pub(super) fn contract_tokens(&self) -> Option<TokenStream> {
        if self.attrs.skip {
            return None;
        }
        // The NAME symbol must be the one the desired view tree actually carries,
        // so it is resolved through the shared `name_binding` registry rather
        // than hashed here (see `name_symbol`/`event_name_symbol`). The TYPE
        // symbol is an opaque type identity that names no view property, so it
        // stays a plain FNV hash.
        let type_sym = Literal::u32_unsuffixed(fnv32(&self.type_string));

        if self.attrs.event {
            let name_sym = Literal::u32_unsuffixed(event_name_symbol(&self.name));
            let mut event = quote! {
                ::obscur_kit::view::contract::EventContract::new(
                    ::obscur_kit::identity::InternedSymbol::from_raw(#name_sym),
                    ::obscur_kit::identity::InternedSymbol::from_raw(#type_sym),
                )
            };
            if self.attrs.bubbles {
                event = quote! { #event.bubbling() };
            }
            if self.attrs.cancelable {
                event = quote! { #event.cancelable() };
            }
            return Some(quote! { .with_event(#event) });
        }

        let name_sym = Literal::u32_unsuffixed(name_symbol(&self.name));
        let optional = self.is_option || self.attrs.default;
        let prop = quote! {
            ::obscur_kit::view::contract::PropContract::new(
                ::obscur_kit::identity::InternedSymbol::from_raw(#name_sym),
                ::obscur_kit::identity::InternedSymbol::from_raw(#type_sym),
            )
        };
        let prop = if optional {
            quote! { #prop.with_default() }
        } else {
            quote! { #prop.required() }
        };
        Some(quote! { .with_prop(#prop) })
    }

    #[doc = include_str!("../../../docs/api/derive/contract_derive.doc/surface_tokens_2.md")]
    pub(super) fn surface_tokens(&self) -> Option<TokenStream> {
        if self.attrs.skip {
            return None;
        }
        let name = LitStr::new(&self.name, proc_macro2::Span::call_site());

        if self.attrs.event {
            return Some(quote! { .allowing_event(#name) });
        }

        let optional = self.is_option || self.attrs.default;
        let prop = if optional {
            quote! { .allowing_prop(#name) }
        } else {
            quote! { .requiring_prop(#name) }
        };
        // The prop's type category, so the `.okui` compiler can reject a literal
        // of the wrong kind (§2.8). An unrecognised type classifies as `Other`
        // and is left unchecked rather than guessed at.
        let variant = syn::Ident::new(classify_prop_type(&self.type_string), Span::call_site());
        Some(quote! {
            #prop.with_prop_type(#name, ::obscur_kit::view::contract::PropTypeSurface::#variant)
        })
    }
}

#[doc = include_str!("../../../docs/api/derive/contract_derive.doc/classify_prop_type.md")]
fn classify_prop_type(type_string: &str) -> &'static str {
    let tokens: Vec<&str> = type_string
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|token| !token.is_empty())
        .collect();
    let has = |name: &str| tokens.iter().any(|token| *token == name);
    if has("String") || has("str") {
        "String"
    } else if has("bool") {
        "Bool"
    } else if has("f32") || has("f64") {
        "Float"
    } else if [
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
    ]
    .iter()
    .any(|int| has(int))
    {
        "Int"
    } else {
        "Other"
    }
}

#[doc = include_str!("../../../docs/api/derive/contract_derive.doc/type_is_option.md")]
fn type_is_option(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    if type_path.qself.is_some() {
        return false;
    }
    type_path
        .path
        .segments
        .last()
        .is_some_and(|seg| seg.ident == "Option")
}

#[doc = include_str!("../../../docs/api/derive/contract_derive.doc/normalized_type_string.md")]
fn normalized_type_string(ty: &Type) -> String {
    let raw = ty.to_token_stream().to_string();
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}
