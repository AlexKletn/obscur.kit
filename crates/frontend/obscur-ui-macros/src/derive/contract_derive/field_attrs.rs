use syn::LitStr;

/// Parsed field-level `#[contract(...)]` flags.
#[derive(Debug, Default)]
pub(super) struct FieldAttrs {
    /// Omit the field from the contract entirely.
    pub(super) skip: bool,
    /// Treat the field as an event rather than a prop.
    pub(super) event: bool,
    /// (Events only) mark the event as bubbling.
    pub(super) bubbles: bool,
    /// (Events only) mark the event as cancelable.
    pub(super) cancelable: bool,
    /// Force the prop to carry a default (`.with_default()` and not required).
    pub(super) default: bool,
    /// Overridden name string used for hashing.
    pub(super) name: Option<String>,
}

impl FieldAttrs {
    /// Parses the field-level `#[contract(...)]` attributes of `field`.
    pub(super) fn parse(field: &syn::Field) -> Result<Self, syn::Error> {
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
