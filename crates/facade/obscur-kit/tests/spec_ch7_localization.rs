//! **SPECIFICATION-DERIVED** localization cases for obscur.kit §7.2.
//!
//! Read this before touching anything in the file.
//!
//! Every expected value below was produced by an agent reading the specification
//! — obscur.kit §7.2 and the CLDR data vendored under `specs/cldr/` — with **no
//! access to this repository's source**. The cases are a statement of what the
//! implementation owes the spec, not a record of what it currently does. A
//! failure here is a defect report about the engine (or, occasionally, about the
//! spec), and must not be "corrected" by editing the expected value.
//!
//! Each test's doc comment carries the quoted normative sentence it encodes.
#![forbid(unsafe_code)]

use obscur_kit::capability::SupportLevel;
use obscur_kit::localization::catalog::{Localizer, MessageCatalog, MessageProvider};
use obscur_kit::localization::locale::LanguageTag;
use obscur_kit::localization::message::{
    GrammaticalGender, MessageArgs, MessageKey, MessagePattern, MessageValue,
};
use obscur_kit::localization::plural::{PluralCategory, PluralOperands, PluralRules};

fn tag(s: &str) -> LanguageTag {
    LanguageTag::parse(s).expect("a well-formed BCP-47 tag")
}

/// §7.2: "Default implementation should be non-Anglocentric and support, at
/// minimum:" / "- Russian plural categories;".
///
/// `specs/cldr/plurals.xml` line 187, verbatim:
/// `<pluralRule count="one">v = 0 and i % 10 = 1 and i % 100 != 11 @integer 1,
/// 21, 31, 41, 51, 61, 71, 81, 101, 1001, …</pluralRule>` and line 189
/// `<pluralRule count="many">v = 0 and i % 10 = 0 or v = 0 and i % 10 = 5..9 or
/// v = 0 and i % 100 = 11..14 @integer 0, 5~19, 100, 1000, …</pluralRule>`.
#[test]
fn ru_partitions_integer_counts_into_one_few_and_many() {
    let rules = PluralRules::cardinal(&tag("ru"));

    let expected = [
        (1_i64, PluralCategory::One),
        (21, PluralCategory::One),
        (101, PluralCategory::One),
        (1001, PluralCategory::One),
        (2, PluralCategory::Few),
        (22, PluralCategory::Few),
        (0, PluralCategory::Many),
        (5, PluralCategory::Many),
        (11, PluralCategory::Many),
        (12, PluralCategory::Many),
        (25, PluralCategory::Many),
        (100, PluralCategory::Many),
        (111, PluralCategory::Many),
    ];

    for (count, category) in expected {
        assert_eq!(
            rules.select_integer(count),
            category,
            "ru plural category of {count}"
        );
    }

    // No integer count in the set may land in `other`: in Russian `other` is
    // reachable only through a visible fraction (see the next case).
    for (count, _) in expected {
        assert_ne!(
            rules.select_integer(count),
            PluralCategory::Other,
            "ru integer {count} must not select `other`"
        );
    }
}

/// §7.2: "- Russian plural categories;".
///
/// `specs/cldr/plurals.xml` line 190, verbatim:
/// `<pluralRule count="other">   @decimal 0.0~1.5, 10.0, 100.0, 1000.0, 10000.0,
/// 100000.0, 1000000.0, …</pluralRule>`, read against the `v = 0` guard on every
/// other `ru` rule.
#[test]
fn ru_other_is_reachable_only_with_a_visible_fraction() {
    let rules = PluralRules::cardinal(&tag("ru"));

    assert_eq!(
        rules.select(&PluralOperands::from_integer(1)),
        PluralCategory::One,
        "1 with no fraction digits is `one`"
    );

    for decimal in ["1.0", "1.5", "21.0", "2.0"] {
        let operands =
            PluralOperands::from_decimal(decimal).expect("a formatted decimal parses to operands");
        assert_eq!(
            rules.select(&operands),
            PluralCategory::Other,
            "ru plural category of the formatted value {decimal}"
        );
    }
}

/// §7.2: "- numeral agreement;" — listed separately from "- Russian plural
/// categories;", so the requirement is agreement of the surrounding message, not
/// just category lookup.
#[test]
fn a_ru_message_agrees_its_noun_with_the_numeral() {
    let mut catalog = MessageCatalog::new();
    catalog
        .insert(
            &tag("ru"),
            "files_selected",
            "{n, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}",
        )
        .expect("a well-formed plural pattern");

    let mut localizer = Localizer::new(tag("ru"));
    localizer.set_provider(Box::new(catalog));

    let render = |count: i64| {
        localizer.format(
            "files_selected",
            &MessageArgs::new().with_integer("n", count),
        )
    };

    let one = render(1);
    let two = render(2);
    let five = render(5);
    let eleven = render(11);

    // The numeral itself appears in each rendering.
    for (count, text) in [(1, &one), (2, &two), (5, &five), (11, &eleven)] {
        assert!(
            text.contains(&count.to_string()),
            "the numeral {count} must appear in its rendering, got {text:?}"
        );
    }

    let noun = |s: &str| {
        s.split_whitespace()
            .last()
            .expect("a rendering has a noun")
            .to_owned()
    };

    assert_ne!(noun(&one), noun(&two), "the noun form differs for 1 and 2");
    assert_ne!(noun(&two), noun(&five), "the noun form differs for 2 and 5");
    assert_ne!(noun(&one), noun(&five), "the noun form differs for 1 and 5");
    assert_eq!(
        noun(&eleven),
        noun(&five),
        "11 takes the same noun form as 5, not the form 1 takes"
    );
    assert_ne!(
        noun(&eleven),
        noun(&one),
        "11 must not take the noun form 1 takes"
    );
}

/// §7.2: "Localization abstraction includes:" / "- parameters;" together with
/// "- gender;".
#[test]
fn gender_selects_the_message_form() {
    let mut catalog = MessageCatalog::new();
    catalog
        .insert(
            &tag("ru"),
            "user_added_file",
            "{g, select, masculine{{name} добавил файл} feminine{{name} добавила файл} \
             other{{name} добавил(а) файл}}",
        )
        .expect("a well-formed select pattern");

    let mut localizer = Localizer::new(tag("ru"));
    localizer.set_provider(Box::new(catalog));

    let masculine = localizer.format(
        "user_added_file",
        &MessageArgs::new()
            .with_text("name", "Саша")
            .with_gender("g", &GrammaticalGender::masculine()),
    );
    let feminine = localizer.format(
        "user_added_file",
        &MessageArgs::new()
            .with_text("name", "Саша")
            .with_gender("g", &GrammaticalGender::feminine()),
    );
    let unmarked = localizer.format(
        "user_added_file",
        &MessageArgs::new().with_text("name", "Саша"),
    );

    assert_ne!(
        masculine, feminine,
        "the past-tense verb form must differ by actor gender"
    );
    assert_ne!(
        unmarked, masculine,
        "an omitted gender must not silently render as masculine"
    );
    assert_ne!(
        unmarked, feminine,
        "an omitted gender must not silently render as feminine"
    );
}

/// A provider that answers only for the **exact** locale a key was registered
/// under — the shape §7.2's fallback bullet is about.
struct ExactProvider {
    entries: Vec<(LanguageTag, MessageKey, MessagePattern)>,
}

impl ExactProvider {
    fn new(rows: &[(&str, &str, &str)]) -> Self {
        Self {
            entries: rows
                .iter()
                .map(|(locale, key, pattern)| {
                    (
                        tag(locale),
                        MessageKey::new(*key),
                        MessagePattern::parse(pattern).expect("a well-formed pattern"),
                    )
                })
                .collect(),
        }
    }
}

impl MessageProvider for ExactProvider {
    fn pattern(&self, locale: &LanguageTag, key: &MessageKey) -> Option<&MessagePattern> {
        self.entries
            .iter()
            .find(|(l, k, _)| l == locale && k == key)
            .map(|(_, _, pattern)| pattern)
    }

    fn availability(&self) -> SupportLevel {
        SupportLevel::Native
    }
}

/// §7.2: "Localization abstraction includes:" / "- current locale;" /
/// "- fallback;".
///
/// WEAKENED 2026-07-19: this originally also asserted that key `C`, absent from
/// both `ru-RU` and `ru`, resolves to `"C-en"` from a DEFAULT LOCALE `en`. §7.2
/// lists "- fallback;" as a bare bullet with no default-locale layer beneath it,
/// the case never configured one, and `Localizer` has no API that could (no
/// `set_default_locale`/`with_fallback_chain` — only `set_locale` and
/// `set_provider`). The two assertions that make the case's own point — that
/// fallback is decided PER KEY, so one format pass can draw `A` from `ru-RU` and
/// `B` from `ru` rather than committing to a single bundle — are kept, and still
/// fail an implementation that picks one bundle and stays in it. Candidate spec
/// amendment: state whether the localizer has a configurable default/root locale
/// beneath the current locale's own chain, and what a key missing from the whole
/// chain returns (the key itself, empty, or a diagnostic).
#[test]
fn fallback_is_resolved_per_key_not_per_bundle() {
    let provider = ExactProvider::new(&[
        ("ru-RU", "A", "A-ru-RU"),
        ("ru", "A", "A-ru"),
        ("ru", "B", "B-ru"),
        ("en", "A", "A-en"),
        ("en", "B", "B-en"),
        ("en", "C", "C-en"),
    ]);

    let mut localizer = Localizer::new(tag("ru-RU"));
    localizer.set_provider(Box::new(provider));

    let args = MessageArgs::new();
    assert_eq!(
        localizer.format("A", &args),
        "A-ru-RU",
        "A comes from ru-RU"
    );
    assert_eq!(
        localizer.format("B", &args),
        "B-ru",
        "B comes from ru — a different source locale from A's, in the same pass, \
         so fallback is decided per key and not per bundle"
    );
}

/// §7.2: "- gender;". Guards the *value* layer the message layer selects on:
/// `specs/cldr/grammaticalFeatures.xml` line 216 gives `ru`
/// `<grammaticalGender values="masculine feminine neuter"/>` and line 220 gives
/// `pl` `<grammaticalGender values="animate inanimate personal feminine neuter"/>`,
/// so a gender value is a *name*, and `personal` must be as expressible as
/// `masculine`.
#[test]
fn a_gender_value_space_is_open_enough_for_polish() {
    let mut catalog = MessageCatalog::new();
    catalog
        .insert(
            &tag("pl"),
            "gone",
            "{g, select, personal{poszedł} feminine{poszła} other{poszło}}",
        )
        .expect("a well-formed select pattern");

    let mut localizer = Localizer::new(tag("pl"));
    localizer.set_provider(Box::new(catalog));

    let personal = localizer.format(
        "gone",
        &MessageArgs::new().with_gender("g", &GrammaticalGender::new("personal")),
    );
    let feminine = localizer.format(
        "gone",
        &MessageArgs::new().with_gender("g", &GrammaticalGender::new("feminine")),
    );
    let neuter = localizer.format(
        "gone",
        &MessageArgs::new().with_gender("g", &GrammaticalGender::new("neuter")),
    );

    assert_eq!(personal, "poszedł", "pl `personal` selects its own arm");
    assert_eq!(feminine, "poszła", "pl `feminine` selects its own arm");
    assert_eq!(
        neuter, "poszło",
        "pl `neuter` falls to the catalogue's declared default arm"
    );
    assert_ne!(
        personal, neuter,
        "personal must not collapse onto a generic masculine/other arm"
    );

    // A gender is carried as a selector value, not as display text.
    let value = MessageValue::gender(&GrammaticalGender::new("inanimate"));
    assert_eq!(
        value.selector_name(),
        "inanimate",
        "pl `inanimate` survives the value layer intact"
    );
}
