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
//!   symbol, the component id, each field's normalized *type* string, and a
//!   §2.12 slot's name — have no view-tree counterpart to agree with, so they
//!   stay plain FNV-1a ([`fnv32`] / [`fnv64`]). A slot name is one of these
//!   because a `slot="header"` reaches the tree as the *value* of a property,
//!   never as a symbol.
//!
//! # Slots (§2.12)
//!
//! A struct-level `#[contract(slot(name = "header", cardinality = one))]`
//! declares one named slot; `cardinality` is `one`/`optional`/`many` (default
//! `many`) and a repeatable `child = "Tag"` restricts what the slot admits.
//!
//! Both views are emitted from the one declaration, and only one of them is read
//! today: [`contract_surface`] is what the `.okui` compiler and the `okui!` macro
//! check a document against, while the typed `contract()`'s `slots` field has no
//! production reader — as is true of its props and events. Emitting it anyway is
//! not decoration: a component that declares a slot and whose typed contract
//! reports `slots: []` would be *wrong*, and an unread field beats a lying one.
//!
//! [`contract_surface`]: ::obscur_view::contract::ComponentSurface

use proc_macro2::{Literal, Span, TokenStream};
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
        let mut field_inits = TokenStream::new();
        for field in &named.named {
            let parsed = FieldModel::parse(field)?;
            if let Some(tokens) = parsed.contract_tokens() {
                entries.extend(tokens);
            }
            if let Some(tokens) = parsed.surface_tokens() {
                surface_entries.extend(tokens);
            }
            field_inits.extend(parsed.from_props_tokens());
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

        // The §2.12 slots, emitted onto both views from the one declaration.
        let slot_tokens: TokenStream = struct_attrs
            .slots
            .iter()
            .map(SlotModel::contract_tokens)
            .collect();
        let surface_slot_tokens: TokenStream = struct_attrs
            .slots
            .iter()
            .map(SlotModel::surface_tokens)
            .collect();

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
                    #slot_tokens
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
                    #surface_slot_tokens
                }
            }

            impl #impl_generics ::obscur_view::component::FromProps
                for #struct_ident #ty_generics #where_clause
            {
                fn from_props(
                    props: &[::obscur_view::view::ViewProperty],
                ) -> ::core::result::Result<Self, ::obscur_view::component::PropsError> {
                    ::core::result::Result::Ok(Self { #field_inits })
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

/// One `#[contract(slot(name = "…", cardinality = …, child = "…"))]` declaration
/// (spec §2.12).
#[derive(Debug)]
struct SlotModel {
    /// The slot's name — the value a child's `slot="…"` must match.
    name: String,
    /// How many children the slot accepts; `many` when unstated, the only
    /// default that adds no obligation the author did not ask for.
    cardinality: SlotCardinality,
    /// The child tags the slot admits; empty means any.
    children: Vec<String>,
}

/// Which `SlotCardinality` variant a `#[contract(slot(cardinality = ...))]`
/// selects.
#[derive(Debug, Clone, Copy)]
enum SlotCardinality {
    /// Exactly one child.
    One,
    /// Zero or one child.
    Optional,
    /// Any number of children.
    Many,
}

impl SlotCardinality {
    /// The `obscur_view::contract::SlotCardinality` variant this spells.
    fn variant(self) -> TokenStream {
        match self {
            Self::One => quote!(ExactlyOne),
            Self::Optional => quote!(Optional),
            Self::Many => quote!(Many),
        }
    }
}

impl SlotModel {
    /// Parses one `slot(...)` list from a struct-level `#[contract(...)]`.
    fn parse(meta: &syn::meta::ParseNestedMeta<'_>) -> Result<Self, syn::Error> {
        let mut name: Option<String> = None;
        let mut cardinality = SlotCardinality::Many;
        let mut children: Vec<String> = Vec::new();
        meta.parse_nested_meta(|inner| {
            if inner.path.is_ident("name") {
                name = Some(inner.value()?.parse::<LitStr>()?.value());
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
                return Err(inner.error("unknown `slot` key (expected `name`, `cardinality` or `child`)"));
            }
            Ok(())
        })?;
        let Some(name) = name else {
            return Err(meta.error("a `slot(...)` must give a `name = \"…\"`"));
        };
        Ok(Self {
            name,
            cardinality,
            children,
        })
    }

    /// Lowers the slot into its `.with_slot(...)` entry on the **typed**
    /// contract.
    ///
    /// Child tags are named by `fnv64`, the derive's own component-id scheme, so
    /// this agrees with `OtherProps::contract().id` — the typed model's internal
    /// spelling of the same tag. It does **not** agree with the `ComponentId` a
    /// front end emits for that tag (`name_binding::component_id_for` resolves
    /// every unregistered tag to `Container`); that disagreement predates slots
    /// and is not made worse here.
    fn contract_tokens(&self) -> TokenStream {
        let name_sym = Literal::u32_unsuffixed(fnv32(&self.name));
        let cardinality = self.cardinality.variant();
        let children = if self.children.is_empty() {
            quote!(::obscur_view::contract::ChildContract::Any)
        } else {
            let ids = self.children.iter().map(|tag| {
                let id = Literal::u64_unsuffixed(fnv64(tag));
                quote!(::obscur_identity::ComponentId::from_raw(#id))
            });
            quote!(::obscur_view::contract::ChildContract::Only(
                ::std::vec![#(#ids),*]
            ))
        };
        quote! {
            .with_slot(::obscur_view::contract::SlotContract::new(
                ::obscur_identity::InternedSymbol::from_raw(#name_sym),
                ::obscur_view::contract::SlotCardinality::#cardinality,
                #children,
            ))
        }
    }

    /// Lowers the slot into its `.with_slot(...)` entry on the **string**
    /// surface, which is the one the `.okui` compiler and the `okui!` macro
    /// actually check a document against.
    fn surface_tokens(&self) -> TokenStream {
        let name = LitStr::new(&self.name, Span::call_site());
        let cardinality = self.cardinality.variant();
        let children = self.children.iter().map(|tag| {
            let tag = LitStr::new(tag, Span::call_site());
            quote!(.allowing_child(#tag))
        });
        quote! {
            .with_slot(
                ::obscur_view::contract::SlotSurface::new(
                    #name,
                    ::obscur_view::contract::SlotCardinality::#cardinality,
                )
                #(#children)*
            )
        }
    }
}

/// Parsed struct-level `#[contract(...)]` configuration.
#[derive(Debug, Default)]
struct StructAttrs {
    /// Overridden component name string (else the struct ident is used).
    name: Option<String>,
    /// Allowed-children rule, if any was specified.
    children: Option<ChildrenKind>,
    /// The declared named slots, in declaration order (spec §2.12).
    slots: Vec<SlotModel>,
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
                } else if meta.path.is_ident("slot") {
                    out.slots.push(SlotModel::parse(&meta)?);
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
            ident: ident.clone(),
            name,
            type_string: normalized_type_string(&field.ty),
            is_option: type_is_option(&field.ty),
            attrs,
        })
    }

    /// Lowers the field into its `<ident>: <expr>` initialiser for the generated
    /// [`FromProps`] impl.
    ///
    /// The three shapes mirror [`contract_tokens`](Self::contract_tokens) exactly,
    /// so what the contract *declares* and what the decoder *reads* cannot drift:
    ///
    /// * a skipped or event field is not a prop at all, so it is not decoded — it
    ///   takes its [`Default`]. (This is why deriving `Contract` on a struct with
    ///   a non-`Default` skipped field is a compile error, which is the honest
    ///   outcome: there is nothing to decode it from.)
    /// * an `Option<T>` decodes optionally — absent or `Null` yields `None`, but a
    ///   present-yet-unreadable value is still an error;
    /// * a `#[contract(default)]` field falls back to its [`Default`] when absent;
    /// * every other field is required.
    ///
    /// The target type is never named: it is inferred from the field being
    /// initialised, so no `Option<T>` inner-type extraction is needed.
    ///
    /// [`FromProps`]: ::obscur_view::component::FromProps
    /// [`Default`]: ::core::default::Default
    fn from_props_tokens(&self) -> TokenStream {
        let ident = &self.ident;

        if self.attrs.skip || self.attrs.event {
            return quote! { #ident: ::core::default::Default::default(), };
        }

        let name_sym = Literal::u32_unsuffixed(name_symbol(&self.name));
        let symbol = quote! { ::obscur_identity::InternedSymbol::from_raw(#name_sym) };

        if self.is_option {
            return quote! {
                #ident: ::obscur_view::component::decode_optional(props, #symbol)?,
            };
        }
        if self.attrs.default {
            return quote! {
                #ident: match ::obscur_view::component::prop_value(props, #symbol) {
                    ::core::option::Option::None
                    | ::core::option::Option::Some(::obscur_view::view::ViewValue::Null) => {
                        ::core::default::Default::default()
                    }
                    ::core::option::Option::Some(value) => {
                        ::obscur_view::component::decode_value(value, #symbol)?
                    }
                },
            };
        }
        quote! {
            #ident: ::obscur_view::component::decode_required(props, #symbol)?,
        }
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
        assert!(
            generated.contains(&format!("from_raw ({disabled})")),
            "{generated}"
        );
        assert!(
            !generated.contains(&format!("from_raw ({})", fnv32("disabled"))),
            "the bare hash must not appear: {generated}"
        );

        // `placeholder` is not reserved: fallback = hash | 0x8000_0000, and its
        // hash does NOT already carry the top bit, so the two really differ.
        let placeholder = name_symbol("placeholder");
        assert_eq!(placeholder, fnv32("placeholder") | 0x8000_0000);
        assert_ne!(placeholder, fnv32("placeholder"));
        assert!(
            generated.contains(&format!("from_raw ({placeholder})")),
            "{generated}"
        );
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
        assert!(
            generated.contains(&format!("from_raw ({click})")),
            "{generated}"
        );
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
    fn a_declared_slot_reaches_both_views_from_one_declaration() {
        let generated = expand_to_string(quote! {
            #[contract(slot(name = "header", cardinality = one))]
            #[contract(slot(name = "actions", cardinality = optional, child = "Button"))]
            struct CardProps {}
        });
        // The string surface — the one the two front ends actually check.
        assert!(
            generated.contains(r#"SlotSurface :: new ("header""#),
            "{generated}"
        );
        assert!(
            generated.contains("SlotCardinality :: ExactlyOne"),
            "{generated}"
        );
        assert!(
            generated.contains(r#"allowing_child ("Button")"#),
            "{generated}"
        );
        // The typed contract, so its `slots` is not a lie.
        assert!(generated.contains("SlotContract :: new"), "{generated}");
        assert!(
            generated.contains("SlotCardinality :: Optional"),
            "{generated}"
        );
        assert!(generated.contains("ChildContract :: Only"), "{generated}");
    }

    #[test]
    fn an_undeclared_cardinality_is_many() {
        let generated = expand_to_string(quote! {
            #[contract(slot(name = "body"))]
            struct CardProps {}
        });
        assert!(generated.contains("SlotCardinality :: Many"), "{generated}");
    }

    #[test]
    fn a_slot_with_no_name_and_an_unknown_key_are_compile_errors() {
        let nameless = ContractDerive::expand(quote! {
            #[contract(slot(cardinality = one))]
            struct CardProps {}
        })
        .expect_err("a slot must be named");
        assert!(nameless.to_string().contains("name"), "{nameless}");

        let unknown = ContractDerive::expand(quote! {
            #[contract(slot(name = "x", cardinaliti = one))]
            struct CardProps {}
        })
        .expect_err("an unknown slot key is a compile error");
        assert!(
            unknown.to_string().contains("unknown `slot` key"),
            "{unknown}"
        );

        let bad_value = ContractDerive::expand(quote! {
            #[contract(slot(name = "x", cardinality = two))]
            struct CardProps {}
        })
        .expect_err("an unknown cardinality is a compile error");
        assert!(bad_value.to_string().contains("two"), "{bad_value}");
    }

    #[test]
    fn a_struct_declaring_no_slots_emits_none() {
        let generated = expand_to_string(quote! { struct MyProps {} });
        assert!(!generated.contains("with_slot"), "{generated}");
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
        // The skipped field contributes no NAME to the surface. It does still
        // appear as an ident in the generated `FromProps` impl — a struct literal
        // must initialise every field — so the check is on the quoted name.
        assert!(!generated.contains(r#""ignore_me""#), "{generated}");
        assert!(generated.contains(r#""keep""#), "{generated}");
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
