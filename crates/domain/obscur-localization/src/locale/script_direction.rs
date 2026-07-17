//! The CLDR tables mapping a script — or a language whose script is implied — to
//! a base writing direction (spec §7.2 RTL integration).
//!
//! Both tables are data, transcribed or derived from pinned CLDR release-48
//! sources under `specs/cldr/`. Neither is a judgement call and neither should be
//! edited by hand: refresh them by re-running the derivation in
//! `specs/cldr/README.md` against a newer release.

use obscur_text::shaping::TextDirection;

/// Every script CLDR marks right-to-left, sorted for binary search.
///
/// Transcribed verbatim from field 6 (`RTL`) of `specs/cldr/scriptMetadata.txt`
/// (CLDR release-48), whose header defines the field as *"YES if the script is
/// RTL — derived from whether the script contains RTL letters according to the
/// Bidi_Class property"*. All 37 of them; the list is exhaustive, not a
/// selection of the popular ones.
static RTL_SCRIPTS: &[&str] = &[
    "Adlm", "Arab", "Armi", "Avst", "Chrs", "Cprt", "Elym", "Gara", "Hatr", "Hebr", "Hung", "Khar",
    "Lydi", "Mand", "Mani", "Mend", "Merc", "Mero", "Narb", "Nbat", "Nkoo", "Orkh", "Ougr", "Palm",
    "Phli", "Phlp", "Phnx", "Prti", "Rohg", "Samr", "Sarb", "Sidt", "Sogd", "Sogo", "Syrc", "Thaa",
    "Yezi",
];

/// Every language whose *likely* script is right-to-left, sorted for binary
/// search.
///
/// This table is what makes a bare `ar` — which is how a locale is actually
/// spelled; nobody writes `ar-Arab` — resolve to
/// [`TextDirection::RightToLeft`]. It is **derived**, not transcribed: it is
/// every bare language subtag in CLDR release-48's `likelySubtags.xml` whose
/// likely script appears in [`RTL_SCRIPTS`]. That file is not vendored (900 KB,
/// and no line of it is quoted); `specs/cldr/README.md` records the pinned URL
/// and the exact derivation.
///
/// All 258 of them, including the deprecated aliases CLDR still carries (`iw`
/// for `he`, `ji` for `yi`, `in` is *not* here because Indonesian is LTR). A
/// hand-picked shortlist would have been the Anglocentric failure in a different
/// costume: it would have covered Arabic, Hebrew and Persian and quietly laid out
/// Sindhi, N'Ko and Dhivehi backwards.
static RTL_LANGUAGES: &[&str] = &[
    "aao", "abh", "abv", "acm", "acq", "acw", "acx", "adf", "ae", "aeb", "aec", "aee", "aeq",
    "afb", "aib", "aii", "aij", "aiq", "amw", "apc", "apd", "ar", "arc", "arq", "ars", "ary",
    "arz", "ask", "atn", "auj", "auz", "avd", "avl", "ayh", "ayl", "ayn", "ayp", "azb", "bal",
    "bdz", "bej", "bft", "bgn", "bgp", "bhe", "bhm", "bhn", "bjf", "bjm", "bqi", "brh", "brk",
    "bsh", "bsk", "chg", "cja", "ckb", "clh", "czk", "dcc", "def", "deh", "dgl", "dmk", "dml",
    "dv", "ecy", "esh", "fa", "fay", "faz", "fia", "fub", "gbz", "ggg", "gha", "ghr", "gig", "gjk",
    "gju", "glh", "glk", "grr", "gwc", "gwf", "gwt", "gzi", "hac", "haz", "hbo", "he", "hkh",
    "hnd", "hno", "hoh", "hrt", "hrz", "hss", "huy", "isk", "itk", "iw", "jad", "jat", "jbe",
    "jbn", "jdg", "ji", "jnd", "jog", "jpa", "jpr", "jrb", "jye", "kbu", "kby", "kcy", "kfm",
    "khw", "klj", "kmz", "kqd", "ks", "ktl", "kvx", "kxp", "lad", "lah", "lhs", "lki", "lrc",
    "lrk", "lrl", "lsa", "lsd", "lss", "luv", "luz", "mby", "mde", "mey", "mfa", "mfi", "mhj",
    "mid", "mki", "mnj", "mve", "mvy", "myz", "mzb", "mzn", "nli", "nlm", "nqo", "ntz", "nyq",
    "oar", "obm", "odk", "oru", "ota", "otk", "oui", "pal", "pbt", "pgd", "phl", "phn", "phr",
    "phv", "plk", "prc", "prd", "prx", "ps", "psh", "psi", "pst", "qxq", "rdb", "rhg", "rmt",
    "sam", "sbn", "scl", "sd", "sdb", "sdf", "sdg", "sdh", "sds", "sgr", "sgy", "shd", "shm",
    "shu", "shv", "siy", "siz", "skr", "smp", "smy", "sog", "sqo", "sqt", "srh", "srz", "ssh",
    "sts", "swb", "syc", "syn", "syr", "tjo", "tks", "tmr", "tov", "tra", "trg", "trm", "trw",
    "ug", "ur", "ush", "uzs", "vaf", "vgr", "vmh", "wbk", "wlo", "wne", "wni", "wsv", "xco", "xhe",
    "xka", "xkc", "xkj", "xkp", "xld", "xly", "xmn", "xmr", "xna", "xpr", "xsa", "xsd", "xvi",
    "ydg", "yhd", "yi", "yih", "yud", "zba", "zdj", "zrp", "zum",
];

/// Returns the base [`TextDirection`] for an ISO-15924 `script` code.
///
/// The comparison is exact and case-sensitive against CLDR's canonical
/// title-case spelling (`Arab`, not `arab`); callers hand this the already
/// canonicalised subtag that
/// [`LanguageTag::parse`](crate::locale::LanguageTag::parse) produced.
#[must_use]
pub fn script_direction(script: &str) -> TextDirection {
    if RTL_SCRIPTS.binary_search(&script).is_ok() {
        TextDirection::RightToLeft
    } else {
        TextDirection::LeftToRight
    }
}

/// Returns the base [`TextDirection`] implied by a `language` subtag alone, for
/// a tag that names no script.
///
/// This is CLDR's likely-script inference collapsed to the one bit this layer
/// needs. An unlisted language answers [`TextDirection::LeftToRight`] — which is
/// a *guess*, and the reason [`LanguageTag::parse`](crate::locale::LanguageTag::parse)
/// keeps an explicit script subtag when the caller supplies one.
#[must_use]
pub fn language_direction(language: &str) -> TextDirection {
    if RTL_LANGUAGES.binary_search(&language).is_ok() {
        TextDirection::RightToLeft
    } else {
        TextDirection::LeftToRight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_are_sorted_so_binary_search_is_valid() {
        // A table that fell out of order would make `binary_search` silently miss
        // entries — an RTL language quietly laid out LTR, with no panic anywhere.
        assert!(RTL_SCRIPTS.windows(2).all(|w| w[0] < w[1]));
        assert!(RTL_LANGUAGES.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn tables_are_the_complete_cldr_sets() {
        // Guards against a "helpful" trim: the counts are what
        // `specs/cldr/scriptMetadata.txt` and the recorded derivation produce.
        assert_eq!(RTL_SCRIPTS.len(), 37);
        assert_eq!(RTL_LANGUAGES.len(), 258);
    }

    #[test]
    fn rtl_scripts_are_rtl_and_others_are_not() {
        assert_eq!(script_direction("Arab"), TextDirection::RightToLeft);
        assert_eq!(script_direction("Hebr"), TextDirection::RightToLeft);
        assert_eq!(script_direction("Thaa"), TextDirection::RightToLeft);
        assert_eq!(script_direction("Latn"), TextDirection::LeftToRight);
        assert_eq!(script_direction("Cyrl"), TextDirection::LeftToRight);
    }

    #[test]
    fn script_lookup_is_case_sensitive_on_the_canonical_spelling() {
        assert_eq!(script_direction("arab"), TextDirection::LeftToRight);
        assert_eq!(script_direction("ARAB"), TextDirection::LeftToRight);
    }

    #[test]
    fn rtl_languages_beyond_the_usual_three() {
        // The shortlist a hand-written table would have stopped at:
        for rtl in ["ar", "he", "fa"] {
            assert_eq!(language_direction(rtl), TextDirection::RightToLeft, "{rtl}");
        }
        // ...and the ones it would have got wrong.
        for rtl in [
            "ur", "ps", "sd", "ug", "yi", "dv", "nqo", "ckb", "azb", "rhg",
        ] {
            assert_eq!(language_direction(rtl), TextDirection::RightToLeft, "{rtl}");
        }
    }

    #[test]
    fn ltr_languages_are_ltr() {
        for ltr in ["en", "ru", "de", "ja", "zh", "hi", "id", "in"] {
            assert_eq!(language_direction(ltr), TextDirection::LeftToRight, "{ltr}");
        }
    }
}
