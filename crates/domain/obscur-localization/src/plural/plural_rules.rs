#![doc = include_str!("plural_rules.doc/_module.md")]

use obscur_capability::SupportLevel;

use crate::locale::LanguageTag;
use crate::plural::{PluralCategory, PluralOperands};

#[doc = include_str!("plural_rules.doc/PluralRules.md")]
#[derive(Debug, Clone, Copy)]
pub struct PluralRules {
    #[doc = include_str!("plural_rules.doc/select.md")]
    select: fn(&PluralOperands) -> PluralCategory,
    #[doc = include_str!("plural_rules.doc/categories.md")]
    categories: &'static [PluralCategory],
    #[doc = include_str!("plural_rules.doc/availability.md")]
    availability: SupportLevel,
}

impl PluralRules {
    #[doc = include_str!("plural_rules.doc/cardinal.md")]
    #[must_use]
    pub fn cardinal(locale: &LanguageTag) -> Self {
        for link in locale.fallback_chain() {
            let name = link.to_string();
            if let Some((_, select, categories)) = RULE_SETS
                .iter()
                .find(|(locales, _, _)| locales.contains(&name.as_str()))
            {
                return Self {
                    select: *select,
                    categories,
                    availability: SupportLevel::Native,
                };
            }
        }
        Self {
            select: root,
            categories: OTHER_ONLY,
            // The root's own rules *are* "everything is other" — asked for `und`,
            // this is a complete answer, not a missing one.
            availability: if locale.is_root() {
                SupportLevel::Native
            } else {
                SupportLevel::Unavailable
            },
        }
    }

    #[doc = include_str!("plural_rules.doc/select_2.md")]
    #[must_use]
    pub fn select(&self, operands: &PluralOperands) -> PluralCategory {
        (self.select)(operands)
    }

    #[doc = include_str!("plural_rules.doc/select_integer.md")]
    #[must_use]
    pub fn select_integer(&self, count: i64) -> PluralCategory {
        self.select(&PluralOperands::from_integer(count))
    }

    #[doc = include_str!("plural_rules.doc/categories_2.md")]
    #[must_use]
    pub fn categories(&self) -> &'static [PluralCategory] {
        self.categories
    }

    #[doc = include_str!("plural_rules.doc/availability_2.md")]
    #[must_use]
    pub const fn availability(&self) -> SupportLevel {
        self.availability
    }
}

#[doc = include_str!("plural_rules.doc/OTHER_ONLY.md")]
const OTHER_ONLY: &[PluralCategory] = &[PluralCategory::Other];

#[doc = include_str!("plural_rules.doc/RULE_SETS.md")]
#[allow(clippy::type_complexity)]
static RULE_SETS: &[(
    &[&str],
    fn(&PluralOperands) -> PluralCategory,
    &'static [PluralCategory],
)] = &[
    (
        &[
            "bm", "bo", "dz", "hnj", "id", "ig", "ii", "in", "ja", "jbo", "jv", "jw", "kde", "kea",
            "km", "ko", "lkt", "lo", "ms", "my", "nqo", "osa", "sah", "ses", "sg", "su", "th",
            "to", "tpi", "vi", "wo", "yo", "yue", "zh",
        ],
        root,
        OTHER_ONLY,
    ),
    (
        &[
            "ast", "de", "en", "et", "fi", "fy", "gl", "ia", "ie", "io", "ji", "lij", "nl", "sc",
            "sv", "sw", "ur", "yi",
        ],
        english,
        &[PluralCategory::One, PluralCategory::Other],
    ),
    (
        &["ru", "uk"],
        russian,
        &[
            PluralCategory::One,
            PluralCategory::Few,
            PluralCategory::Many,
            PluralCategory::Other,
        ],
    ),
    (
        &["pl"],
        polish,
        &[
            PluralCategory::One,
            PluralCategory::Few,
            PluralCategory::Many,
            PluralCategory::Other,
        ],
    ),
    (
        &["cs", "sk"],
        czech,
        &[
            PluralCategory::One,
            PluralCategory::Few,
            PluralCategory::Many,
            PluralCategory::Other,
        ],
    ),
    (&["ar", "ars"], arabic, &PluralCategory::ALL),
];

#[doc = include_str!("plural_rules.doc/root.md")]
fn root(_: &PluralOperands) -> PluralCategory {
    PluralCategory::Other
}

#[doc = include_str!("plural_rules.doc/english.md")]
fn english(op: &PluralOperands) -> PluralCategory {
    if op.i == 1 && op.v == 0 {
        PluralCategory::One
    } else {
        PluralCategory::Other
    }
}

#[doc = include_str!("plural_rules.doc/russian.md")]
fn russian(op: &PluralOperands) -> PluralCategory {
    if op.v != 0 {
        // All three named categories require `v = 0`; a written fraction can only
        // be `other`.
        return PluralCategory::Other;
    }
    let i10 = op.i % 10;
    let i100 = op.i % 100;
    if i10 == 1 && i100 != 11 {
        PluralCategory::One
    } else if (2..=4).contains(&i10) && !(12..=14).contains(&i100) {
        PluralCategory::Few
    } else if i10 == 0 || (5..=9).contains(&i10) || (11..=14).contains(&i100) {
        PluralCategory::Many
    } else {
        PluralCategory::Other
    }
}

#[doc = include_str!("plural_rules.doc/polish.md")]
fn polish(op: &PluralOperands) -> PluralCategory {
    if op.v != 0 {
        return PluralCategory::Other;
    }
    let i10 = op.i % 10;
    let i100 = op.i % 100;
    if op.i == 1 {
        PluralCategory::One
    } else if (2..=4).contains(&i10) && !(12..=14).contains(&i100) {
        PluralCategory::Few
    } else if (op.i != 1 && (0..=1).contains(&i10))
        || (5..=9).contains(&i10)
        || (12..=14).contains(&i100)
    {
        PluralCategory::Many
    } else {
        PluralCategory::Other
    }
}

#[doc = include_str!("plural_rules.doc/czech.md")]
fn czech(op: &PluralOperands) -> PluralCategory {
    if op.i == 1 && op.v == 0 {
        PluralCategory::One
    } else if (2..=4).contains(&op.i) && op.v == 0 {
        PluralCategory::Few
    } else if op.v != 0 {
        PluralCategory::Many
    } else {
        PluralCategory::Other
    }
}

#[doc = include_str!("plural_rules.doc/arabic.md")]
fn arabic(op: &PluralOperands) -> PluralCategory {
    if op.n == 0.0 {
        return PluralCategory::Zero;
    }
    if op.n == 1.0 {
        return PluralCategory::One;
    }
    if op.n == 2.0 {
        return PluralCategory::Two;
    }
    // `n % 100` against an integer range: only an `n` with no visible fraction can
    // land in it (TR35 ranges enumerate integers).
    let Some(n100) = integer_modulus(op.n, 100) else {
        return PluralCategory::Other;
    };
    if (3..=10).contains(&n100) {
        PluralCategory::Few
    } else if (11..=99).contains(&n100) {
        PluralCategory::Many
    } else {
        PluralCategory::Other
    }
}

#[doc = include_str!("plural_rules.doc/integer_modulus.md")]
fn integer_modulus(n: f64, modulus: u64) -> Option<u64> {
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
        return None;
    }
    // `n` reached here from `PluralOperands`, whose `n` is an absolute value with
    // no fractional part; the cast is exact for every count that can occur.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let whole = n as u64;
    Some(whole % modulus)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plural::PluralCategory::{Few, Many, One, Other, Two, Zero};

    fn rules(tag: &str) -> PluralRules {
        PluralRules::cardinal(&LanguageTag::parse(tag).expect("valid tag"))
    }

    /// Asserts every integer in `samples` selects `expected` under `rules`.
    fn assert_integers(rules: &PluralRules, expected: PluralCategory, samples: &[i64]) {
        for &n in samples {
            assert_eq!(
                rules.select_integer(n),
                expected,
                "{n} should be `{expected}`"
            );
        }
    }

    // ---- Russian: the rule §7.2 names ------------------------------------

    #[test]
    fn russian_matches_its_cldr_integer_samples() {
        // Transcribed from the `@integer` lists of `<pluralRules locales="ru uk">`
        // in `specs/cldr/plurals.xml`, expanded (`2~4` -> 2, 3, 4).
        let ru = rules("ru");
        assert_integers(&ru, One, &[1, 21, 31, 41, 51, 61, 71, 81, 101, 1001]);
        assert_integers(
            &ru,
            Few,
            &[
                2, 3, 4, 22, 23, 24, 32, 33, 34, 42, 43, 44, 52, 53, 54, 62, 102, 1002,
            ],
        );
        assert_integers(
            &ru,
            Many,
            &[
                0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 100, 1000, 10_000,
                100_000, 1_000_000,
            ],
        );
    }

    #[test]
    fn russian_keys_on_the_last_two_digits_not_the_last_one() {
        let ru = rules("ru");
        // Every one of these ends in the digit 1. A rule that looked at the last
        // digit alone would call all five `one`.
        assert_eq!(ru.select_integer(1), One);
        assert_eq!(ru.select_integer(21), One);
        assert_eq!(ru.select_integer(101), One);
        assert_eq!(ru.select_integer(11), Many);
        assert_eq!(ru.select_integer(111), Many);
        // ...and these all end in 2.
        assert_eq!(ru.select_integer(2), Few);
        assert_eq!(ru.select_integer(22), Few);
        assert_eq!(ru.select_integer(12), Many);
        assert_eq!(ru.select_integer(112), Many);
    }

    #[test]
    fn russian_written_fractions_are_other() {
        // The `v = 0` guard on all three named categories.
        let ru = rules("ru");
        for source in ["1.0", "1.5", "21.0", "2.5", "0.0"] {
            let op = PluralOperands::from_decimal(source).expect(source);
            assert_eq!(ru.select(&op), Other, "{source}");
        }
    }

    #[test]
    fn ukrainian_shares_the_russian_rules() {
        // Same CLDR group; the registry, not a coincidence.
        let uk = rules("uk");
        assert_eq!(uk.select_integer(21), One);
        assert_eq!(uk.select_integer(111), Many);
    }

    #[test]
    fn russian_is_reached_through_the_fallback_chain() {
        let ru_ru = rules("ru-RU");
        assert_eq!(ru_ru.availability(), SupportLevel::Native);
        assert_eq!(ru_ru.select_integer(21), One);
        assert_eq!(rules("ru-Cyrl-RU").select_integer(111), Many);
    }

    // ---- the other transcribed groups ------------------------------------

    #[test]
    fn english_matches_its_cldr_integer_samples() {
        let en = rules("en");
        assert_integers(&en, One, &[1]);
        assert_integers(
            &en,
            Other,
            &[0, 2, 3, 4, 16, 100, 1000, 10_000, 100_000, 1_000_000],
        );
    }

    #[test]
    fn english_one_is_not_n_equals_one() {
        let en = rules("en");
        let op = PluralOperands::from_decimal("1.0").expect("valid");
        assert_eq!(en.select(&op), Other);
    }

    #[test]
    fn polish_matches_its_cldr_integer_samples() {
        let pl = rules("pl");
        assert_integers(&pl, One, &[1]);
        assert_integers(
            &pl,
            Few,
            &[
                2, 3, 4, 22, 23, 24, 32, 33, 34, 42, 43, 44, 52, 53, 54, 62, 102, 1002,
            ],
        );
        assert_integers(
            &pl,
            Many,
            &[
                0, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 100, 1000, 10_000,
                100_000, 1_000_000,
            ],
        );
    }

    #[test]
    fn polish_is_not_russian() {
        // Four categories each; the same number, different answers. This is the
        // assertion that fails if the two rules are ever "unified".
        let (ru, pl) = (rules("ru"), rules("pl"));
        assert_eq!(ru.select_integer(21), One);
        assert_eq!(pl.select_integer(21), Many);
        assert_eq!(ru.select_integer(101), One);
        assert_eq!(pl.select_integer(101), Many);
        assert_eq!(ru.select_integer(11), Many);
        assert_eq!(pl.select_integer(11), Many);
    }

    #[test]
    fn czech_matches_its_cldr_integer_samples() {
        let cs = rules("cs");
        assert_integers(&cs, One, &[1]);
        assert_integers(&cs, Few, &[2, 3, 4]);
        assert_integers(
            &cs,
            Other,
            &[0, 5, 6, 10, 19, 100, 1000, 10_000, 100_000, 1_000_000],
        );
        // `many` is Czech's *fraction* category — nothing integral reaches it.
        for source in ["0.0", "1.5", "10.0", "1000.0"] {
            let op = PluralOperands::from_decimal(source).expect(source);
            assert_eq!(cs.select(&op), Many, "{source}");
        }
    }

    #[test]
    fn czech_is_neither_russian_nor_polish() {
        let (ru, pl, cs) = (rules("ru"), rules("pl"), rules("cs"));
        // 5: Russian and Polish say `many`, Czech says `other`.
        assert_eq!(ru.select_integer(5), Many);
        assert_eq!(pl.select_integer(5), Many);
        assert_eq!(cs.select_integer(5), Other);
        // 22: Russian and Polish say `few`, Czech says `other`.
        assert_eq!(ru.select_integer(22), Few);
        assert_eq!(pl.select_integer(22), Few);
        assert_eq!(cs.select_integer(22), Other);
    }

    #[test]
    fn arabic_matches_its_cldr_integer_samples() {
        let ar = rules("ar");
        assert_integers(&ar, Zero, &[0]);
        assert_integers(&ar, One, &[1]);
        assert_integers(&ar, Two, &[2]);
        assert_integers(&ar, Few, &[3, 4, 5, 6, 7, 8, 9, 10, 103, 110, 1003]);
        assert_integers(&ar, Many, &[11, 12, 20, 26, 111, 1011]);
        assert_integers(
            &ar,
            Other,
            &[100, 101, 102, 200, 201, 202, 600, 1000, 10_000, 1_000_000],
        );
    }

    #[test]
    fn arabic_keys_on_n_so_a_written_fraction_can_still_be_one() {
        // The counterpart of `russian_written_fractions_are_other`: the rules are
        // written against different operands, and it shows.
        let ar = rules("ar");
        for source in ["1.0", "1.00", "1.0000"] {
            let op = PluralOperands::from_decimal(source).expect(source);
            assert_eq!(ar.select(&op), One, "{source}");
        }
        // ...but a real fraction matches no integer range.
        let op = PluralOperands::from_decimal("3.5").expect("valid");
        assert_eq!(ar.select(&op), Other);
    }

    #[test]
    fn japanese_has_one_category() {
        let ja = rules("ja");
        assert_eq!(ja.availability(), SupportLevel::Native);
        assert_eq!(ja.categories(), OTHER_ONLY);
        assert_integers(&ja, Other, &[0, 1, 2, 5, 15, 100, 1_000_000]);
    }

    // ---- honest capability ------------------------------------------------

    #[test]
    fn root_is_native_and_other_only() {
        let root = PluralRules::cardinal(&LanguageTag::root());
        assert_eq!(root.availability(), SupportLevel::Native);
        assert_eq!(root.select_integer(1), Other);
    }

    #[test]
    fn an_untranscribed_locale_reports_unavailable_rather_than_guessing() {
        // French's CLDR rules are written against the compact-decimal exponent
        // operand this crate does not model, so they are deliberately absent.
        // The point is that it *says so* — and that it does not quietly answer
        // with English's rules.
        let fr = rules("fr");
        assert_eq!(fr.availability(), SupportLevel::Unavailable);
        assert_eq!(fr.select_integer(1), Other);
        assert_ne!(fr.select_integer(1), rules("en").select_integer(1));
    }

    #[test]
    fn categories_report_what_each_rule_set_can_produce() {
        assert_eq!(rules("en").categories(), [One, Other]);
        assert_eq!(rules("ru").categories(), [One, Few, Many, Other]);
        assert_eq!(rules("ar").categories(), PluralCategory::ALL);
    }

    #[test]
    fn every_rule_set_only_produces_the_categories_it_advertises() {
        // A rule whose branches disagree with its advertised `categories()` would
        // let a catalogue believe it had covered every arm when it had not.
        for tag in ["ja", "en", "ru", "uk", "pl", "cs", "sk", "ar"] {
            let rules = rules(tag);
            for n in -5i64..2000 {
                let category = rules.select_integer(n);
                assert!(
                    rules.categories().contains(&category),
                    "{tag}: {n} selected `{category}`, not advertised"
                );
            }
            for whole in 0..200 {
                for frac in ["0", "5", "00", "50", "25"] {
                    let source = format!("{whole}.{frac}");
                    let op = PluralOperands::from_decimal(&source).expect(&source);
                    let category = rules.select(&op);
                    assert!(
                        rules.categories().contains(&category),
                        "{tag}: {source} selected `{category}`, not advertised"
                    );
                }
            }
        }
    }

    #[test]
    fn the_root_is_never_a_registered_rule_set() {
        // The invariant `cardinal`'s "nothing matched" tail rests on: every
        // fallback chain ends at the root, so registering it would make *every*
        // locale report `Native`. CLDR does list `root` in the one-category
        // group's `locales`, so transcribing that element faithfully is exactly
        // how the bug gets in.
        for (locales, _, _) in RULE_SETS {
            for locale in *locales {
                assert!(
                    !matches!(*locale, "und" | "root"),
                    "`{locale}` is the root locale and must not be a rule-set key"
                );
            }
        }
        // ...and the behaviour it protects, for a locale with no rules of its own.
        assert_eq!(rules("fr").availability(), SupportLevel::Unavailable);
    }

    #[test]
    fn rule_set_locale_lists_are_disjoint() {
        // CLDR lists each locale in exactly one group; a duplicate here would make
        // `cardinal` depend on registry order.
        let mut seen: Vec<&str> = RULE_SETS
            .iter()
            .flat_map(|(l, _, _)| l.iter().copied())
            .collect();
        let total = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), total, "a locale is registered in two rule sets");
    }
}
