#![doc = include_str!("../../docs/api/locale/script_direction.doc/_module.md")]

use obscur_text::shaping::TextDirection;

#[doc = include_str!("../../docs/api/locale/script_direction.doc/RTL_SCRIPTS.md")]
static RTL_SCRIPTS: &[&str] = &[
    "Adlm", "Arab", "Armi", "Avst", "Chrs", "Cprt", "Elym", "Gara", "Hatr", "Hebr", "Hung", "Khar",
    "Lydi", "Mand", "Mani", "Mend", "Merc", "Mero", "Narb", "Nbat", "Nkoo", "Orkh", "Ougr", "Palm",
    "Phli", "Phlp", "Phnx", "Prti", "Rohg", "Samr", "Sarb", "Sidt", "Sogd", "Sogo", "Syrc", "Thaa",
    "Yezi",
];

#[doc = include_str!("../../docs/api/locale/script_direction.doc/RTL_LANGUAGES.md")]
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

#[doc = include_str!("../../docs/api/locale/script_direction.doc/script_direction.md")]
#[must_use]
pub fn script_direction(script: &str) -> TextDirection {
    if RTL_SCRIPTS.binary_search(&script).is_ok() {
        TextDirection::RightToLeft
    } else {
        TextDirection::LeftToRight
    }
}

#[doc = include_str!("../../docs/api/locale/script_direction.doc/language_direction.md")]
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
