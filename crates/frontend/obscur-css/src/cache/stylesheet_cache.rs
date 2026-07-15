//! `StylesheetCache`: parse each CSS source once per revision (spec §4.12).

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::parser::{ParsedStylesheet, StylesheetParser};
use crate::source::{CssSource, SourceId, SourceVersion};

/// Caches the parse of each CSS source, keyed by its identity **and** revision
/// (spec §4.12: "CSS parsing is cached per unique source/version and invalidated
/// explicitly").
///
/// # What decides a hit
///
/// The `(SourceId, SourceVersion)` pair, never the text. A source whose owner has
/// not bumped its version is the same source, and re-parsing it would be exactly
/// the work this exists to avoid — including re-reading the very text a hash would
/// have to walk. An [`ANONYMOUS`](SourceId::ANONYMOUS) source is deliberately
/// **not** cached: it has no stable identity, so two unrelated anonymous sheets
/// would collide under one key and the second would silently get the first's rules.
///
/// # Explicit invalidation
///
/// Two ways, both the owner's choice: bump the source's
/// [`version`](CssSource::version) (the entry no longer matches), or call
/// [`invalidate`](Self::invalidate)/[`clear`](Self::clear). Nothing here watches
/// files or times anything out — §4.12 asks for explicit invalidation, and a cache
/// that expires entries on its own would be neither explicit nor predictable.
#[derive(Debug, Default)]
pub struct StylesheetCache {
    /// One entry per identified source: the revision it was parsed at, and the
    /// parse. A new revision replaces the entry rather than growing the map.
    entries: HashMap<SourceId, (SourceVersion, ParsedStylesheet)>,
    /// Scratch slot for the most recent ANONYMOUS parse.
    ///
    /// An anonymous source is re-parsed every call (it has no identity to cache
    /// by), but [`parse`](Self::parse) hands back a reference, so the result has to
    /// live somewhere. It is deliberately a single overwritten slot rather than a
    /// map entry: keying anonymous sources would make two unrelated sheets collide,
    /// and the second would silently receive the first's rules.
    anonymous: Option<ParsedStylesheet>,
}

impl StylesheetCache {
    /// Creates an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The cached parse of `source`, or [`None`] on a miss — an unseen source, a
    /// stale revision, or an anonymous source (never cached).
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

    /// Returns `source`'s parse, parsing it only on a miss.
    ///
    /// This is the whole point of the cache, so it is the method to reach for: a
    /// caller that parses first and inserts second has already paid the cost the
    /// cache exists to skip.
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

    /// Drops `id`'s entry, if any, returning whether one was cached.
    ///
    /// The explicit half of §4.12's invalidation, for an owner that changed a
    /// sheet without a new [`CssSource`] to stamp.
    pub fn invalidate(&mut self, id: SourceId) -> bool {
        self.entries.remove(&id).is_some()
    }

    /// Drops every entry.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// How many sources are cached.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is cached.
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
