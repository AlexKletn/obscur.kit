#![doc = include_str!("font_fallback.doc/_module.md")]

use crate::font::{FontFamily, GenericFamily};

#[doc = include_str!("font_fallback.doc/FontFallback.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontFallback {
    families: Vec<FontFamily>,
}

impl FontFallback {
    #[doc = include_str!("font_fallback.doc/new.md")]
    #[must_use]
    pub fn new(primary: FontFamily) -> Self {
        Self {
            families: vec![primary],
        }
    }

    #[doc = include_str!("font_fallback.doc/from_families.md")]
    #[must_use]
    pub fn from_families(families: Vec<FontFamily>) -> Self {
        let mut chain = Self {
            families: Vec::with_capacity(families.len()),
        };
        for family in families {
            chain.push(family);
        }
        chain
    }

    #[doc = include_str!("font_fallback.doc/default_chain.md")]
    #[must_use]
    pub fn default_chain() -> Self {
        Self::from_families(vec![
            FontFamily::generic(GenericFamily::SystemUi),
            FontFamily::generic(GenericFamily::SansSerif),
        ])
    }

    #[doc = include_str!("font_fallback.doc/push.md")]
    pub fn push(&mut self, family: FontFamily) {
        if !self.families.contains(&family) {
            self.families.push(family);
        }
    }

    #[doc = include_str!("font_fallback.doc/with.md")]
    #[must_use]
    pub fn with(mut self, family: FontFamily) -> Self {
        self.push(family);
        self
    }

    #[doc = include_str!("font_fallback.doc/families.md")]
    #[must_use]
    pub fn families(&self) -> &[FontFamily] {
        &self.families
    }

    #[doc = include_str!("font_fallback.doc/primary.md")]
    #[must_use]
    pub fn primary(&self) -> Option<&FontFamily> {
        self.families.first()
    }

    #[doc = include_str!("font_fallback.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, family: &FontFamily) -> bool {
        self.families.contains(family)
    }

    #[doc = include_str!("font_fallback.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.families.len()
    }

    #[doc = include_str!("font_fallback.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.families.is_empty()
    }

    #[doc = include_str!("font_fallback.doc/iter.md")]
    pub fn iter(&self) -> std::slice::Iter<'_, FontFamily> {
        self.families.iter()
    }
}

impl<'a> IntoIterator for &'a FontFallback {
    type Item = &'a FontFamily;
    type IntoIter = std::slice::Iter<'a, FontFamily>;

    fn into_iter(self) -> Self::IntoIter {
        self.families.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_holds_single_primary() {
        let chain = FontFallback::new(FontFamily::named("Inter"));
        assert_eq!(chain.len(), 1);
        assert!(!chain.is_empty());
        assert_eq!(chain.primary(), Some(&FontFamily::named("Inter")));
    }

    #[test]
    fn push_keeps_primary_first_and_dedups() {
        let mut chain = FontFallback::new(FontFamily::named("Inter"));
        chain.push(FontFamily::generic(GenericFamily::SansSerif));
        // Pushing the existing primary again is a no-op.
        chain.push(FontFamily::named("Inter"));
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.primary(), Some(&FontFamily::named("Inter")));
        assert_eq!(
            chain.families(),
            &[
                FontFamily::named("Inter"),
                FontFamily::generic(GenericFamily::SansSerif),
            ]
        );
    }

    #[test]
    fn with_is_fluent_and_dedups() {
        let chain = FontFallback::new(FontFamily::named("Inter"))
            .with(FontFamily::named("Roboto"))
            .with(FontFamily::named("Roboto"));
        assert_eq!(chain.len(), 2);
        assert!(chain.contains(&FontFamily::named("Roboto")));
    }

    #[test]
    fn from_families_dedups_preserving_order() {
        let chain = FontFallback::from_families(vec![
            FontFamily::named("A"),
            FontFamily::named("B"),
            FontFamily::named("A"),
        ]);
        assert_eq!(
            chain.families(),
            &[FontFamily::named("A"), FontFamily::named("B")]
        );
    }

    #[test]
    fn default_chain_ends_in_sans_serif() {
        let chain = FontFallback::default_chain();
        assert_eq!(
            chain.families().last(),
            Some(&FontFamily::generic(GenericFamily::SansSerif))
        );
    }

    #[test]
    fn iter_yields_in_order() {
        let chain = FontFallback::new(FontFamily::named("A")).with(FontFamily::named("B"));
        let names: Vec<&str> = chain.iter().map(FontFamily::name).collect();
        assert_eq!(names, ["A", "B"]);
    }
}
