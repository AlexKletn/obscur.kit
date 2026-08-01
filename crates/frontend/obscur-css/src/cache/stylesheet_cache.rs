#![doc = include_str!("stylesheet_cache.doc/_module.md")]

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::parser::{ParsedStylesheet, StylesheetParser};
use crate::source::{CssSource, SourceId, SourceVersion};

#[doc = include_str!("stylesheet_cache.doc/StylesheetCache.md")]
#[derive(Debug, Default)]
pub struct StylesheetCache {
    #[doc = include_str!("stylesheet_cache.doc/entries.md")]
    entries: HashMap<SourceId, (SourceVersion, ParsedStylesheet)>,
    #[doc = include_str!("stylesheet_cache.doc/anonymous.md")]
    anonymous: Option<ParsedStylesheet>,
}

impl StylesheetCache {
    #[doc = include_str!("stylesheet_cache.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("stylesheet_cache.doc/get.md")]
    #[must_use]
    pub fn get(&self, source: &CssSource) -> Option<&ParsedStylesheet> {
        if source.id() == SourceId::ANONYMOUS {
            return None;
        }
        self.entries
            .get(&source.id())
            .filter(|(version, _)| *version == source.version())
            .map(|(_, parsed)| parsed)
    }

    #[doc = include_str!("stylesheet_cache.doc/parse.md")]
    pub fn parse(&mut self, source: &CssSource) -> &ParsedStylesheet {
        // An anonymous source is parsed every time and never stored — see the
        // type-level note on why it must not share a key.
        if source.id() == SourceId::ANONYMOUS {
            return self.anonymous.insert(StylesheetParser::new(source).parse());
        }
        match self.entries.entry(source.id()) {
            // A hit: same source, same revision. This is the branch the cache
            // exists for — no parse, no text touched.
            Entry::Occupied(entry) if entry.get().0 == source.version() => &entry.into_mut().1,
            // A stale revision: re-parse and replace, so the map holds one entry
            // per source rather than growing with every edit.
            Entry::Occupied(mut entry) => {
                entry.insert((source.version(), StylesheetParser::new(source).parse()));
                &entry.into_mut().1
            }
            Entry::Vacant(entry) => {
                &entry
                    .insert((source.version(), StylesheetParser::new(source).parse()))
                    .1
            }
        }
    }

    #[doc = include_str!("stylesheet_cache.doc/invalidate.md")]
    pub fn invalidate(&mut self, id: SourceId) -> bool {
        self.entries.remove(&id).is_some()
    }

    #[doc = include_str!("stylesheet_cache.doc/clear.md")]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    #[doc = include_str!("stylesheet_cache.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[doc = include_str!("stylesheet_cache.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::StylesheetCache;
    use crate::source::{CssSource, SourceId, SourceVersion};

    fn sheet(text: &str) -> CssSource {
        CssSource::new(SourceId::from_raw(1), "app.css", text)
    }

    /// The first selector text of the parse's first rule — enough to tell one
    /// sheet's rules from another's.
    fn first_rule_len(cache: &mut StylesheetCache, source: &CssSource) -> usize {
        cache.parse(source).rules.len()
    }

    #[test]
    fn an_unchanged_source_is_parsed_once() {
        // THE point of §4.12: the same source at the same revision must not be
        // re-parsed. Identity is proven by mutating the cached entry and observing
        // that the second call returns the MUTATED value — impossible if it had
        // re-parsed from the text.
        let mut cache = StylesheetCache::new();
        let source = sheet(".a { color: red } .b { color: blue }");
        assert_eq!(first_rule_len(&mut cache, &source), 2);
        assert_eq!(cache.len(), 1);

        // Reach in and truncate the cached parse.
        cache
            .entries
            .get_mut(&source.id())
            .expect("cached")
            .1
            .rules
            .truncate(1);

        assert_eq!(
            first_rule_len(&mut cache, &source),
            1,
            "the second call returned the cached parse, not a fresh one"
        );
    }

    #[test]
    fn a_new_revision_invalidates_the_entry() {
        let mut cache = StylesheetCache::new();
        let first = sheet(".a { color: red }");
        assert_eq!(first_rule_len(&mut cache, &first), 1);

        // The owner edits the sheet and stamps the next revision.
        let second = CssSource::new(SourceId::from_raw(1), "app.css", ".a {} .b {} .c {}")
            .at_version(SourceVersion::INITIAL.next());
        assert!(cache.get(&second).is_none(), "a new revision is a miss");
        assert_eq!(first_rule_len(&mut cache, &second), 3, "it re-parsed");
        assert_eq!(cache.len(), 1, "the new revision replaced the old entry");
    }

    #[test]
    fn invalidate_and_clear_drop_entries_explicitly() {
        let mut cache = StylesheetCache::new();
        let source = sheet(".a {}");
        let _ = first_rule_len(&mut cache, &source);

        assert!(cache.invalidate(source.id()), "the entry was cached");
        assert!(!cache.invalidate(source.id()), "and is gone now");
        assert!(cache.get(&source).is_none());

        let _ = first_rule_len(&mut cache, &source);
        assert!(!cache.is_empty());
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn two_anonymous_sources_never_share_a_parse() {
        // An anonymous source has no stable identity, so caching it by id would
        // make unrelated sheets collide — the second silently receiving the first's
        // rules. It is re-parsed every time instead, and stored nowhere.
        let mut cache = StylesheetCache::new();
        let one = CssSource::anonymous(".a {}");
        let two = CssSource::anonymous(".a {} .b {} .c {}");

        assert_eq!(first_rule_len(&mut cache, &one), 1);
        assert_eq!(
            first_rule_len(&mut cache, &two),
            3,
            "the second anonymous sheet parsed its OWN text"
        );
        assert_eq!(first_rule_len(&mut cache, &one), 1, "and back again");
        assert!(cache.is_empty(), "no anonymous source is ever cached");
        assert!(cache.get(&one).is_none());
    }

    #[test]
    fn distinct_sources_get_distinct_entries() {
        let mut cache = StylesheetCache::new();
        let a = CssSource::new(SourceId::from_raw(1), "a.css", ".a {}");
        let b = CssSource::new(SourceId::from_raw(2), "b.css", ".b {} .c {}");
        assert_eq!(first_rule_len(&mut cache, &a), 1);
        assert_eq!(first_rule_len(&mut cache, &b), 2);
        assert_eq!(cache.len(), 2);
    }
}
