#![doc = include_str!("spec_ch4b_css_language.doc/_module.md")]

mod spec_ch4b_support;

use obscur_kit::css::cache::StylesheetCache;
use obscur_kit::css::source::{CssSource, SourceId};

#[doc = include_str!("spec_ch4b_css_language.doc/ch4_12_a_parse_is_cached_per_source_and_explicitly_invalidated.md")]
#[test]
fn ch4_12_a_parse_is_cached_per_source_and_explicitly_invalidated() {
    const S: &str = ".card { color: red; } .inner { color: blue; }";
    let id = SourceId::from_raw(7);
    let source = CssSource::new(id, "shared.css", S);

    let mut cache = StylesheetCache::new();

    // Apply to scope A: an unseen source, so a miss that parses.
    assert!(
        cache.get(&source).is_none(),
        "the first apply must miss: nothing is cached yet"
    );
    let handle_a = std::ptr::from_ref(cache.parse(&source));

    // Apply to scope B: the byte-identical source at the same version.
    assert!(
        cache.get(&source).is_some(),
        "the second apply of a byte-identical source must be a cache hit"
    );
    let handle_b = std::ptr::from_ref(cache.parse(&source));
    assert_eq!(
        handle_a, handle_b,
        "scopes A and B must share the same parsed-sheet handle — S is parsed \
         exactly once across the two applies"
    );
    assert_eq!(cache.len(), 1, "one source, one cache entry");

    // The explicit invalidation entry point the clause requires.
    assert!(
        cache.invalidate(id),
        "the explicit invalidation API must report that an entry was dropped"
    );
    assert!(
        cache.get(&source).is_none(),
        "after explicit invalidation the third apply must be a miss and reparse"
    );
    let _ = cache.parse(&source);
    assert_eq!(
        cache.len(),
        1,
        "the third apply reparses and re-populates the single entry"
    );
}
