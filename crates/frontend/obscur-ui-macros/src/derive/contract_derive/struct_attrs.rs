use syn::{DeriveInput, LitStr};

use super::{children_kind::ChildrenKind, slot_model::SlotModel, style_scope_kind::StyleScopeKind};

/// Parsed struct-level `#[contract(...)]` configuration.
#[derive(Debug, Default)]
pub(super) struct StructAttrs {
    /// Overridden component name string (else the struct ident is used).
    pub(super) name: Option<String>,
    /// Allowed-children rule, if any was specified.
    pub(super) children: Option<ChildrenKind>,
    /// The declared named slots, in declaration order (spec §2.12).
    pub(super) slots: Vec<SlotModel>,
    /// The declared §4.15 style scope, if the author stated one.
    pub(super) style_scope: Option<StyleScopeKind>,
}

impl StructAttrs {
    /// Parses the struct-level `#[contract(...)]` attributes of `input`.
    pub(super) fn parse(input: &DeriveInput) -> Result<Self, syn::Error> {
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
                } else if meta.path.is_ident("style_scope") {
                    let value = meta.value()?;
                    let lit: syn::Ident = value.parse()?;
                    out.style_scope = Some(match lit.to_string().as_str() {
                        "scoped" => StyleScopeKind::Scoped,
                        "open" => StyleScopeKind::Open,
                        other => {
                            return Err(meta.error(format!(
                                "unknown `style_scope` value `{other}` (expected `scoped` or \
                                 `open`)"
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
