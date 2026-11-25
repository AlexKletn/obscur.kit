//! Resolves a text node's computed `font-family` into a font fallback chain:
//! [`families_for`].

use obscur_style::computed::ComputedStyle;
use obscur_style::property::PropertyId;
use obscur_text::font::{FontFallback, FontFamily};

#[doc = include_str!("../../docs/api/orchestration/font_families.doc/families_for.md")]
#[must_use]
pub(crate) fn families_for(computed: Option<&ComputedStyle>) -> FontFallback {
    let names = computed
        .and_then(|style| style.get(PropertyId::FontFamily))
        .and_then(|value| value.value().as_font_families());
    match names {
        Some(list) if !list.is_empty() => FontFallback::from_families(
            list.iter()
                .map(|name| FontFamily::from_name(name))
                .collect(),
        ),
        _ => FontFallback::default_chain(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_style::computed::ComputedValue;
    use obscur_style::property::PropertyValue;
    use obscur_text::font::GenericFamily;

    #[test]
    fn absent_font_family_is_the_default_chain() {
        // No computed style at all, and a computed style without `font-family`,
        // both fall back to the default chain (unchanged behaviour).
        assert_eq!(families_for(None), FontFallback::default_chain());
        let style = ComputedStyle::new();
        assert_eq!(families_for(Some(&style)), FontFallback::default_chain());
    }

    #[test]
    fn empty_font_family_list_is_the_default_chain() {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::FontFamily,
            ComputedValue::from_concrete(PropertyValue::font_families(Vec::<String>::new()))
                .expect("font families are concrete"),
        );
        assert_eq!(families_for(Some(&style)), FontFallback::default_chain());
    }

    #[test]
    fn authored_font_family_maps_to_the_chain_in_order() {
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::FontFamily,
            ComputedValue::from_concrete(PropertyValue::font_families([
                "Foo",
                "Arial",
                "sans-serif",
            ]))
            .expect("font families are concrete"),
        );
        let expected = FontFallback::from_families(vec![
            FontFamily::from_name("Foo"),
            FontFamily::from_name("Arial"),
            FontFamily::generic(GenericFamily::SansSerif),
        ]);
        assert_eq!(families_for(Some(&style)), expected);
        // The last entry resolved to the generic sans-serif family, not a name.
        assert_eq!(
            expected.families().last(),
            Some(&FontFamily::generic(GenericFamily::SansSerif))
        );
    }
}
