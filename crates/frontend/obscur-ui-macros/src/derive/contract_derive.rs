//! `ContractDerive`: expansion logic for `#[derive(Contract)]` (spec §2.13).
//!
//! This module turns a props struct into an inherent `impl` exposing a
//! `contract()` associated function that builds a fully-typed
//! [`::obscur_view::component::ComponentContract`]. The macro crate emits
//! leading-`::` paths as tokens, so the *generated* code depends on
//! `obscur-view`/`obscur-identity` while this crate does not.
//!
//! # Stable-id scheme
//!
//! [`::obscur_identity::InternedSymbol`] and
//! [`::obscur_identity::ComponentId`] are opaque `from_raw` handles, and there
//! is no compile-time interner available during derive expansion, so every id is
//! computed at expansion time and only the resulting integer literal is emitted.
//!
//! There are two *different* schemes here, and the distinction is load-bearing:
//!
//! - **Names that also exist in the desired view tree** — each prop/event field
//!   name — are resolved through the shared `obscur_ui_ir::name_binding`
//!   registry ([`name_symbol`] / [`event_name_symbol`]), because a contract's
//!   prop symbol is only useful if it equals the `ViewProperty::name()` a real
//!   node carries. `name_binding` maps reserved names (`class`, `id`, `type`,
//!   `src`, `disabled`, `checked`; `click`, `input`, `change`) to fixed low
//!   constants and everything else to `fnv32(name) | 0x8000_0000`. A bare
//!   `fnv32` — which this derive used to emit — agrees with that only by luck,
//!   whenever the hash happens to already carry the top bit.
//! - **Names that exist only inside the contract model** — the component name
//!   symbol, the component id, and each field's normalized *type* string — have
//!   no view-tree counterpart to agree with, so they stay plain FNV-1a
//!   ([`fnv32`] / [`fnv64`]).

use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, quote};
use syn::{Data, DeriveInput, Fields, LitStr, Type, spanned::Spanned};

/// Expander for `#[derive(Contract)]`.
///
/// Call [`ContractDerive::expand`] with the annotated item's tokens to obtain
/// the generated inherent `impl` (or a [`syn::Error`] for unsupported inputs).
#[derive(Debug)]
pub(crate) struct ContractDerive;

impl ContractDerive {
    /// Expands `#[derive(Contract)]` on a props struct into an inherent impl
    /// producing an obscur-view [`::obscur_view::component::ComponentContract`].
    ///
    /// Only structs with named fields are supported; enums, unions and
    /// tuple/unit structs produce a [`syn::Error`] at the input span. Unknown
    /// `#[contract(...)]` keys are also errors (spec: unknown attributes are
    /// compile errors).
    ///
    /// # Errors
    ///
    /// Returns a [`syn::Error`] when the input cannot be parsed as a
    /// [`syn::DeriveInput`], when the item is not a named-field struct, or when
    /// an unrecognised `#[contract(...)]` attribute or value is encountered.
    pub(crate) fn expand(input: TokenStream) -> Result<TokenStream, syn::Error> {
        let derive_input: DeriveInput = syn::parse2(input)?;
        let struct_ident = &derive_input.ident;
        let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

        let Data::Struct(data) = &derive_input.data else {
            return Err(syn::Error::new(
                derive_input.span(),
                "Contract can only be derived for structs with named fields",
            ));
        };
        let Fields::Named(named) = &data.fields else {
            return Err(syn::Error::new(
                derive_input.span(),
                "Contract can only be derived for structs with named fields",
            ));
        };

        // Struct-level attributes: component name + allowed children.
        let struct_attrs = StructAttrs::parse(&derive_input)?;
        let component_name = struct_attrs
            .name
            .unwrap_or_else(|| struct_ident.to_string());
        let component_id = Literal::u64_unsuffixed(fnv64(&component_name));
        let name_sym = Literal::u32_unsuffixed(fnv32(&component_name));
        let component_name_literal = LitStr::new(&component_name, derive_input.ident.span());

        // Per-field entries for both the typed contract (symbol-keyed) and the
        // string tooling surface, skipping `#[contract(skip)]`.
        let mut entries = TokenStream::new();
        let mut surface_entries = TokenStream::new();
        for field in &named.named {
            let parsed = FieldModel::parse(field)?;
            if let Some(tokens) = parsed.contract_tokens() {
                entries.extend(tokens);
            }
            if let Some(tokens) = parsed.surface_tokens() {
                surface_entries.extend(tokens);
            }
        }

        let children_tokens = struct_attrs.children.map(|kind| {
            let variant = match kind {
                ChildrenKind::Any => quote!(Any),
                ChildrenKind::None => quote!(None),
            };
            quote! {
                .allowing_children(::obscur_view::contract::ChildContract::#variant)
            }
        });

        // The parallel string surface's children rule mirrors the typed one.
        let surface_children_tokens = struct_attrs.children.map(|kind| match kind {
            ChildrenKind::Any => quote!(.any_children()),
            ChildrenKind::None => quote!(.no_children()),
        });

        Ok(quote! {
            impl #impl_generics #struct_ident #ty_generics #where_clause {
                /// Returns the typed component contract for this props struct,
                /// generated by `#[derive(Contract)]`.
                #[must_use]
                pub fn contract() -> ::obscur_view::component::ComponentContract {
                    ::obscur_view::component::ComponentContract::new(
                        ::obscur_identity::ComponentId::from_raw(#component_id)
                    )
                    .with_name(::obscur_identity::InternedSymbol::from_raw(#name_sym))
                    #entries
                    #children_tokens
                }

                /// Returns the string-keyed tooling surface for this props
                /// struct, generated by `#[derive(Contract)]` from the same
                /// names as [`contract`](Self::contract) before they are hashed.
                ///
                /// This is the view a string-keyed tool such as the `.okui`
                /// compiler consumes to enforce §2.13 for this component; the
                /// typed [`contract`](Self::contract) remains the symbol-keyed
                /// runtime view.
                #[must_use]
                pub fn contract_surface() -> ::obscur_view::contract::ComponentSurface {
                    ::obscur_view::contract::ComponentSurface::new(#component_name_literal)
                    #surface_entries
                    #surface_children_tokens
                }
            }
        })
    }
}

/// Which `ChildContract` variant a `#[contract(children = ...)]` selects.
#[derive(Debug, Clone, Copy)]
enum ChildrenKind {
    /// Any child component is accepted.
    Any,
    /// No children are accepted.
    None,
}

/// Parsed struct-level `#[contract(...)]` configuration.
#[derive(Debug, Default)]
struct StructAttrs {
    /// Overridden component name string (else the struct ident is used).
    name: Option<String>,
    /// Allowed-children rule, if any was specified.
    children: Option<ChildrenKind>,
}

impl StructAttrs {
    /// Parses the struct-level `#[contract(...)]` attributes of `input`.
    fn parse(input: &DeriveInput) -> Result<Self, syn::Error> {
        let mut out = Self::default();
        for attr in &input.attrs {
            if !attr.path().is_ident("contract") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;
                    out.name = Some(lit.value());
                    Ok(())
                } else if meta.path.is_ident("children") {
                    let value = meta.value()?;
                    let lit: syn::Ident = value.parse()?;
                    out.children = Some(match lit.to_string().as_str() {
                        "any" => ChildrenKind::Any,
                        "none" => ChildrenKind::None,
                        other => {
                            return Err(meta.error(format!(
                                "unknown `children` value `{other}` (expected `any` or `none`)"
                            )));
                        }
                    });
                    Ok(())
                } else {
                    Err(meta.error("unknown `contract` attribute on struct"))
                }
            })?;
        }
        Ok(out)
    }
}

/// The parsed model of a single struct field for contract generation.
#[derive(Debug)]
struct FieldModel {
    /// The name string used for hashing (field ident or `name = "..."` override).
    name: String,
    /// The normalized textual type string used for the payload/value-type hash.
    type_string: String,
    /// Whether the field is an `Option<...>` (implies optional + default).
    is_option: bool,
    /// Field-level attribute flags.
    attrs: FieldAttrs,
}

/// Parsed field-level `#[contract(...)]` flags.
#[derive(Debug, Default)]
struct FieldAttrs {
    /// Omit the field from the contract entirely.
    skip: bool,
    /// Treat the field as an event rather than a prop.
    event: bool,
    /// (Events only) mark the event as bubbling.
    bubbles: bool,
    /// (Events only) mark the event as cancelable.
    cancelable: bool,
    /// Force the prop to carry a default (`.with_default()` and not required).
    default: bool,
    /// Overridden name string used for hashing.
    name: Option<String>,
}

impl FieldModel {
    /// Parses a named struct `field` into a [`FieldModel`].
    fn parse(field: &syn::Field) -> Result<Self, syn::Error> {
        let ident = field
            .ident
            .as_ref()
            .expect("named struct fields always carry an ident");
        let attrs = FieldAttrs::parse(field)?;
        let name = attrs.name.clone().unwrap_or_else(|| ident.to_string());
        Ok(Self {
            name,
            type_string: normalized_type_string(&field.ty),
            is_option: type_is_option(&field.ty),
            attrs,
        })
    }

    /// Lowers the field into its typed `.with_prop(...)` / `.with_event(...)`
    /// builder call, or `None` when the field is skipped.
    fn contract_tokens(&self) -> Option<TokenStream> {
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
                ::obscur_view::contract::EventContract::new(
                    ::obscur_identity::InternedSymbol::from_raw(#name_sym),
                    ::obscur_identity::InternedSymbol::from_raw(#type_sym),
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

        // Props: an `Option<...>` or an explicit `#[contract(default)]` become a
        // defaulted (non-required) prop; every other field is required.
        let name_sym = Literal::u32_unsuffixed(name_symbol(&self.name));
        let optional = self.is_option || self.attrs.default;
        let prop = quote! {
            ::obscur_view::contract::PropContract::new(
                ::obscur_identity::InternedSymbol::from_raw(#name_sym),
                ::obscur_identity::InternedSymbol::from_raw(#type_sym),
            )
        };
        let prop = if optional {
            quote! { #prop.with_default() }
        } else {
            quote! { #prop.required() }
        };
        Some(quote! { .with_prop(#prop) })
    }

    /// Lowers the field into its string-surface builder call
    /// (`.requiring_prop(...)` / `.allowing_prop(...)` / `.allowing_event(...)`),
    /// or `None` when the field is skipped.
    ///
    /// The name is emitted as the raw string literal (before it is hashed for
    /// the typed contract), so a string-keyed tool can match it against `.okui`
    /// attribute/event names. Required vs. allowed mirrors
    /// [`contract_tokens`](Self::contract_tokens): an `Option<...>` or
    /// `#[contract(default)]` prop is allowed (optional); every other prop is
    /// required; an event field becomes an allowed event.
    fn surface_tokens(&self) -> Option<TokenStream> {
        if self.attrs.skip {
            return None;
        }
        let name = LitStr::new(&self.name, proc_macro2::Span::call_site());

        if self.attrs.event {
            return Some(quote! { .allowing_event(#name) });
        }

        let optional = self.is_option || self.attrs.default;
        if optional {
            Some(quote! { .allowing_prop(#name) })
        } else {
            Some(quote! { .requiring_prop(#name) })
        }
    }
}

impl FieldAttrs {
    /// Parses the field-level `#[contract(...)]` attributes of `field`.
    fn parse(field: &syn::Field) -> Result<Self, syn::Error> {
        let mut out = Self::default();
        for attr in &field.attrs {
            if !attr.path().is_ident("contract") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    out.skip = true;
                    Ok(())
                } else if meta.path.is_ident("event") {
                    out.event = true;
                    Ok(())
                } else if meta.path.is_ident("bubbles") {
                    out.bubbles = true;
                    Ok(())
                } else if meta.path.is_ident("cancelable") {
                    out.cancelable = true;
                    Ok(())
                } else if meta.path.is_ident("default") {
                    out.default = true;
                    Ok(())
                } else if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;
                    out.name = Some(lit.value());
                    Ok(())
                } else {
                    Err(meta.error("unknown `contract` attribute on field"))
                }
            })?;
        }
        Ok(out)
    }
}

/// Returns whether `ty` is syntactically an `Option<...>`.
///
/// This is a purely textual check on the last path segment, matching `Option`,
/// `std::option::Option` and `core::option::Option`.
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

/// Returns the whitespace-normalized textual form of `ty`.
///
/// `ToTokens::to_token_stream().to_string()` already renders a single space
/// between tokens; this collapses any runs and trims the ends so equal types
/// hash identically regardless of the caller's source formatting.
fn normalized_type_string(ty: &Type) -> String {
    let raw = ty.to_token_stream().to_string();
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// FNV-1a 32-bit hash of `s`, used to derive stable [`InternedSymbol`] ids.
/// The [`InternedSymbol`] raw value a **property** name has in the desired view
/// tree, resolved through the shared `name_binding` registry.
///
/// This must not be a bare hash. `name_binding::attr_symbol` maps a handful of
/// reserved names (`class`, `id`, `type`, `src`, `disabled`, `checked`) to fixed
/// low constants, and every other name to `fnv32(name) | 0x8000_0000`. A bare
/// `fnv32` therefore disagrees with the real view symbol for every reserved name
/// and for every other name whose hash does not already have the top bit set —
/// i.e. it happens to agree about half the time, by luck. Resolving through the
/// registry is what makes a contract's prop symbol comparable to the
/// `ViewProperty::name()` a node actually carries.
///
/// [`InternedSymbol`]: obscur_identity::InternedSymbol
fn name_symbol(name: &str) -> u32 {
    obscur_ui_ir::name_binding::attr_symbol(name).raw()
}

/// The [`InternedSymbol`] raw value an **event** name has in the desired view
/// tree, resolved through the shared `name_binding` registry — the event-side
/// mirror of [`name_symbol`], and reserved for `click`/`input`/`change`.
///
/// [`InternedSymbol`]: obscur_identity::InternedSymbol
fn event_name_symbol(name: &str) -> u32 {
    obscur_ui_ir::name_binding::event_symbol(name).raw()
}

fn fnv32(s: &str) -> u32 {
    const OFFSET: u32 = 0x811c_9dc5;
    const PRIME: u32 = 0x0100_0193;
    let mut hash = OFFSET;
    for byte in s.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// FNV-1a 64-bit hash of `s`, used to derive stable [`ComponentId`] ids.
fn fnv64(s: &str) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET;
    for byte in s.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::{ContractDerive, event_name_symbol, fnv32, fnv64, name_symbol};
    use quote::quote;

    /// Collapses generated tokens to a single-spaced string for tolerant
    /// substring assertions.
    fn expand_to_string(input: proc_macro2::TokenStream) -> String {
        ContractDerive::expand(input)
            .expect("expansion should succeed")
            .to_string()
    }

    #[test]
    fn required_and_option_props_are_distinguished() {
        let generated = expand_to_string(quote! {
            struct MyProps {
                label: String,
                title: Option<String>,
            }
        });
        assert!(generated.contains("ComponentContract :: new"));
        assert_eq!(generated.matches("with_prop").count(), 2);
        assert_eq!(generated.matches(". required ()").count(), 1);
        assert_eq!(generated.matches(". with_default ()").count(), 1);
    }

    #[test]
    fn explicit_default_attr_makes_prop_defaulted() {
        let generated = expand_to_string(quote! {
            struct MyProps {
                #[contract(default)]
                count: u32,
            }
        });
        assert_eq!(generated.matches(". with_default ()").count(), 1);
        assert_eq!(generated.matches(". required ()").count(), 0);
    }

    #[test]
    fn event_field_emits_event_contract() {
        let generated = expand_to_string(quote! {
            struct MyProps {
                #[contract(event)]
                on_click: ClickPayload,
            }
        });
        assert!(generated.contains("with_event"));
        assert!(generated.contains("EventContract :: new"));
        assert_eq!(generated.matches("with_prop").count(), 0);
    }

    #[test]
    fn event_bubbles_and_cancelable_chain() {
        let generated = expand_to_string(quote! {
            struct MyProps {
                #[contract(event, bubbles, cancelable)]
                on_click: ClickPayload,
            }
        });
        assert!(generated.contains(". bubbling ()"));
        assert!(generated.contains(". cancelable ()"));
    }

    #[test]
    fn event_without_flags_has_no_propagation_calls() {
        let generated = expand_to_string(quote! {
            struct MyProps {
                #[contract(event)]
                on_click: ClickPayload,
            }
        });
        assert!(!generated.contains(". bubbling ()"));
        assert!(!generated.contains(". cancelable ()"));
    }

    #[test]
    fn skip_omits_the_field() {
        let generated = expand_to_string(quote! {
            struct MyProps {
                keep: String,
                #[contract(skip)]
                ignore_me: String,
            }
        });
        assert_eq!(generated.matches("with_prop").count(), 1);
    }

    #[test]
    fn struct_name_override_changes_component_id_literal() {
        let default_named = expand_to_string(quote! { struct MyProps {} });
        let overridden = expand_to_string(quote! {
            #[contract(name = "Button")]
            struct MyProps {}
        });
        let button_id = fnv64("Button");
        let button_name = fnv32("Button");
        assert!(overridden.contains(&format!("from_raw ({button_id})")));
        assert!(overridden.contains(&format!("from_raw ({button_name})")));
        assert!(!default_named.contains(&format!("from_raw ({button_id})")));
    }

    #[test]
    fn field_name_override_changes_symbol_literal() {
        let generated = expand_to_string(quote! {
            struct MyProps {
                #[contract(name = "customName")]
                original: String,
            }
        });
        // Resolved through `name_binding`, NOT a bare hash — see `name_symbol`.
        let custom_sym = name_symbol("customName");
        let original_sym = name_symbol("original");
        assert!(generated.contains(&format!("from_raw ({custom_sym})")));
        assert!(!generated.contains(&format!("from_raw ({original_sym})")));
    }

    #[test]
    fn prop_name_symbols_are_the_ones_the_view_tree_carries() {
        // The bug this pins: a bare `fnv32` disagrees with the real view symbol
        // for every RESERVED name, and for any other name whose hash lacks the
        // top bit — so it matched only about half the time, by luck. A contract
        // whose prop symbol does not equal `ViewProperty::name()` can never be
        // checked against a real node.
        let generated = expand_to_string(quote! {
            struct MyProps {
                disabled: bool,
                placeholder: String,
            }
        });

        // `disabled` is reserved: a low fixed constant, not a hash at all.
        let disabled = name_symbol("disabled");
        assert_eq!(disabled, 0x0000_0005, "reserved DISABLED symbol");
        assert!(generated.contains(&format!("from_raw ({disabled})")), "{generated}");
        assert!(
            !generated.contains(&format!("from_raw ({})", fnv32("disabled"))),
            "the bare hash must not appear: {generated}"
        );

        // `placeholder` is not reserved: fallback = hash | 0x8000_0000, and its
        // hash does NOT already carry the top bit, so the two really differ.
        let placeholder = name_symbol("placeholder");
        assert_eq!(placeholder, fnv32("placeholder") | 0x8000_0000);
        assert_ne!(placeholder, fnv32("placeholder"));
        assert!(generated.contains(&format!("from_raw ({placeholder})")), "{generated}");
    }

    #[test]
    fn event_name_symbols_use_the_reserved_event_handles() {
        let generated = expand_to_string(quote! {
            struct MyProps {
                #[contract(event)]
                click: u32,
            }
        });
        // `click` is a reserved event symbol, not a hash.
        let click = event_name_symbol("click");
        assert!(generated.contains(&format!("from_raw ({click})")), "{generated}");
    }

    #[test]
    fn children_any_and_none_emit_allowing_children() {
        let any = expand_to_string(quote! {
            #[contract(children = any)]
            struct MyProps {}
        });
        assert!(any.contains("allowing_children"));
        assert!(any.contains("ChildContract :: Any"));

        let none = expand_to_string(quote! {
            #[contract(children = none)]
            struct MyProps {}
        });
        assert!(none.contains("ChildContract :: None"));
    }

    #[test]
    fn no_children_attr_omits_allowing_children() {
        let generated = expand_to_string(quote! { struct MyProps {} });
        assert!(!generated.contains("allowing_children"));
    }

    #[test]
    fn emits_contract_surface_alongside_typed_contract() {
        let generated = expand_to_string(quote! {
            #[contract(name = "Tabs")]
            struct TabsProps {
                selected: String,
                orientation: Option<String>,
                #[contract(event)]
                on_change: ChangePayload,
            }
        });
        // Both associated functions are emitted.
        assert!(generated.contains("fn contract ()"));
        assert!(generated.contains("fn contract_surface ()"));
        assert!(generated.contains("ComponentSurface :: new"));
        // The surface keeps the raw string names (they are hashed away in the
        // typed contract).
        assert!(generated.contains("\"Tabs\""));
        assert!(generated.contains(". requiring_prop (\"selected\")"));
        assert!(generated.contains(". allowing_prop (\"orientation\")"));
        assert!(generated.contains(". allowing_event (\"on_change\")"));
    }

    #[test]
    fn surface_children_mirror_the_children_attr() {
        let any = expand_to_string(quote! {
            #[contract(children = any)]
            struct MyProps {}
        });
        assert!(any.contains(". any_children ()"));

        let none = expand_to_string(quote! {
            #[contract(children = none)]
            struct MyProps {}
        });
        assert!(none.contains(". no_children ()"));

        let default = expand_to_string(quote! { struct MyProps {} });
        assert!(!default.contains("any_children"));
        assert!(!default.contains("no_children"));
    }

    #[test]
    fn surface_omits_skipped_fields() {
        let generated = expand_to_string(quote! {
            struct MyProps {
                keep: String,
                #[contract(skip)]
                ignore_me: String,
            }
        });
        // One prop in the surface (skipped field absent).
        assert_eq!(generated.matches("requiring_prop").count(), 1);
        assert!(!generated.contains("ignore_me"));
    }

    #[test]
    fn surface_field_name_override_is_used_verbatim() {
        let generated = expand_to_string(quote! {
            struct MyProps {
                #[contract(name = "customName")]
                original: String,
            }
        });
        assert!(generated.contains(". requiring_prop (\"customName\")"));
        assert!(!generated.contains("\"original\""));
    }

    #[test]
    fn enum_input_is_an_error() {
        let result = ContractDerive::expand(quote! {
            enum NotAStruct { A, B }
        });
        assert!(result.is_err());
    }

    #[test]
    fn tuple_struct_is_an_error() {
        let result = ContractDerive::expand(quote! {
            struct Tuple(u32, String);
        });
        assert!(result.is_err());
    }

    #[test]
    fn unknown_struct_attr_key_is_an_error() {
        let result = ContractDerive::expand(quote! {
            #[contract(bogus)]
            struct MyProps {}
        });
        assert!(result.is_err());
    }

    #[test]
    fn unknown_field_attr_key_is_an_error() {
        let result = ContractDerive::expand(quote! {
            struct MyProps {
                #[contract(bogus)]
                field: String,
            }
        });
        assert!(result.is_err());
    }

    #[test]
    fn fnv_helpers_are_deterministic_and_distinct() {
        assert_eq!(fnv32("hello"), fnv32("hello"));
        assert_ne!(fnv32("hello"), fnv32("world"));
        assert_eq!(fnv64("hello"), fnv64("hello"));
        assert_ne!(fnv64("hello"), fnv64("world"));
        // Empty string is the bare FNV offset basis.
        assert_eq!(fnv32(""), 0x811c_9dc5);
        assert_eq!(fnv64(""), 0xcbf2_9ce4_8422_2325);
    }
}
