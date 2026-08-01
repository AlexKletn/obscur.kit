#![doc = include_str!("localizer.doc/_module.md")]

use obscur_capability::SupportLevel;
use obscur_text::shaping::TextDirection;

use crate::catalog::{EmptyMessageProvider, MessageProvider};
use crate::locale::LanguageTag;
use crate::message::{MessageArgs, MessageKey};
use crate::plural::PluralRules;

#[doc = include_str!("localizer.doc/Localizer.md")]
pub struct Localizer {
    #[doc = include_str!("localizer.doc/locale.md")]
    locale: LanguageTag,
    #[doc = include_str!("localizer.doc/rules.md")]
    rules: PluralRules,
    #[doc = include_str!("localizer.doc/provider.md")]
    provider: Box<dyn MessageProvider>,
}

impl Localizer {
    #[doc = include_str!("localizer.doc/new.md")]
    #[must_use]
    pub fn new(locale: LanguageTag) -> Self {
        Self {
            rules: PluralRules::cardinal(&locale),
            locale,
            provider: Box::new(EmptyMessageProvider),
        }
    }

    #[doc = include_str!("localizer.doc/with_provider.md")]
    #[must_use]
    pub fn with_provider(mut self, provider: Box<dyn MessageProvider>) -> Self {
        self.provider = provider;
        self
    }

    #[doc = include_str!("localizer.doc/set_provider.md")]
    pub fn set_provider(&mut self, provider: Box<dyn MessageProvider>) {
        self.provider = provider;
    }

    #[doc = include_str!("localizer.doc/locale_2.md")]
    #[must_use]
    pub fn locale(&self) -> &LanguageTag {
        &self.locale
    }

    #[doc = include_str!("localizer.doc/set_locale.md")]
    pub fn set_locale(&mut self, locale: LanguageTag) {
        self.rules = PluralRules::cardinal(&locale);
        self.locale = locale;
    }

    #[doc = include_str!("localizer.doc/plural_rules.md")]
    #[must_use]
    pub fn plural_rules(&self) -> &PluralRules {
        &self.rules
    }

    #[doc = include_str!("localizer.doc/direction.md")]
    #[must_use]
    pub fn direction(&self) -> TextDirection {
        self.locale.direction()
    }

    #[doc = include_str!("localizer.doc/language.md")]
    #[must_use]
    pub fn language(&self) -> &str {
        self.locale.language()
    }

    #[doc = include_str!("localizer.doc/availability.md")]
    #[must_use]
    pub fn availability(&self) -> SupportLevel {
        self.provider.availability()
    }

    #[doc = include_str!("localizer.doc/format.md")]
    #[must_use]
    pub fn format(&self, key: impl Into<MessageKey>, args: &MessageArgs) -> String {
        let key = key.into();
        for link in self.locale.fallback_chain() {
            if let Some(pattern) = self.provider.pattern(&link, &key) {
                return pattern.format(args, &self.rules);
            }
        }
        key.as_str().to_owned()
    }
}

impl Default for Localizer {
    #[doc = include_str!("localizer.doc/default.md")]
    fn default() -> Self {
        Self::new(LanguageTag::root())
    }
}

impl std::fmt::Debug for Localizer {
    #[doc = include_str!("localizer.doc/fmt.md")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Localizer")
            .field("locale", &self.locale)
            .field("availability", &self.provider.availability())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::MessageCatalog;
    use crate::message::GrammaticalGender;
    use crate::plural::PluralCategory;

    fn tag(s: &str) -> LanguageTag {
        LanguageTag::parse(s).expect("valid tag")
    }

    /// A catalogue with the Russian file-count message registered under `ru`.
    fn russian_catalog() -> MessageCatalog {
        let mut catalog = MessageCatalog::new();
        catalog
            .insert(
                &tag("ru"),
                "files",
                "{n, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}",
            )
            .expect("valid pattern");
        catalog
    }

    #[test]
    fn defaults_to_root_over_an_empty_provider() {
        let localizer = Localizer::default();
        assert!(localizer.locale().is_root());
        assert_eq!(localizer.availability(), SupportLevel::Unavailable);
        assert_eq!(localizer.direction(), TextDirection::LeftToRight);
    }

    #[test]
    fn a_missing_key_renders_as_itself() {
        let localizer = Localizer::new(tag("ru"));
        assert_eq!(
            localizer.format("file.count", &MessageArgs::new()),
            "file.count"
        );
    }

    #[test]
    fn formats_through_the_fallback_chain() {
        // Registered under `ru`; asked for as `ru-RU`.
        let localizer = Localizer::new(tag("ru-RU")).with_provider(Box::new(russian_catalog()));
        let files = |n| localizer.format("files", &MessageArgs::new().with_integer("n", n));
        assert_eq!(files(1), "1 файл");
        assert_eq!(files(5), "5 файлов");
    }

    #[test]
    fn russian_numeral_agreement_end_to_end() {
        // §7.2's minimum, through the whole abstraction — key, parameters, locale,
        // fallback, provider — and chosen to discriminate:
        //
        //   1, 2, 5   alone would pass under a rule that looks at the last digit;
        //   21        ends in 1 and is `one`, so a "> 4 is plural" rule fails here;
        //   111       ends in 1 *and* has `i % 10 == 1`, and is `many`, so only a
        //             rule reading the last *two* digits gets it right.
        let localizer = Localizer::new(tag("ru-RU")).with_provider(Box::new(russian_catalog()));
        let files = |n| localizer.format("files", &MessageArgs::new().with_integer("n", n));

        assert_eq!(files(1), "1 файл");
        assert_eq!(files(2), "2 файла");
        assert_eq!(files(5), "5 файлов");
        assert_eq!(files(21), "21 файл");
        assert_eq!(files(111), "111 файлов");
    }

    #[test]
    fn the_locale_decides_the_rules_not_the_catalogue() {
        // Same catalogue entry, formatted by a localizer whose locale is `en`:
        // English rules, so 2 is `other`, and the `few` arm is unreachable.
        let mut catalog = russian_catalog();
        catalog
            .insert(
                &tag("en"),
                "files",
                "{n, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}",
            )
            .expect("valid pattern");
        let en = Localizer::new(tag("en")).with_provider(Box::new(catalog));
        assert_eq!(
            en.format("files", &MessageArgs::new().with_integer("n", 2)),
            "2 файла"
        );
        assert_eq!(
            en.format("files", &MessageArgs::new().with_integer("n", 5)),
            "5 файла"
        );
    }

    #[test]
    fn switching_locale_switches_the_rules_with_it() {
        let mut localizer = Localizer::new(tag("ru"));
        assert_eq!(
            localizer.plural_rules().select_integer(21),
            PluralCategory::One
        );
        localizer.set_locale(tag("pl"));
        assert_eq!(
            localizer.plural_rules().select_integer(21),
            PluralCategory::Many
        );
        localizer.set_locale(tag("en"));
        assert_eq!(
            localizer.plural_rules().select_integer(21),
            PluralCategory::Other
        );
    }

    #[test]
    fn switching_locale_switches_the_direction_and_the_shaping_language() {
        let mut localizer = Localizer::new(tag("ru-RU"));
        assert_eq!(localizer.direction(), TextDirection::LeftToRight);
        assert_eq!(localizer.language(), "ru");
        localizer.set_locale(tag("ar-EG"));
        assert_eq!(localizer.direction(), TextDirection::RightToLeft);
        assert_eq!(localizer.language(), "ar");
    }

    #[test]
    fn the_provider_is_replaceable() {
        let mut localizer = Localizer::new(tag("ru"));
        assert_eq!(localizer.availability(), SupportLevel::Unavailable);
        assert_eq!(
            localizer.format("files", &MessageArgs::new().with_integer("n", 1)),
            "files"
        );
        localizer.set_provider(Box::new(russian_catalog()));
        assert_eq!(localizer.availability(), SupportLevel::Native);
        assert_eq!(
            localizer.format("files", &MessageArgs::new().with_integer("n", 1)),
            "1 файл"
        );
    }

    #[test]
    fn numeral_agreement_gender_and_a_name_in_one_message() {
        // What §7.2's minimum looks like in one string.
        let mut catalog = MessageCatalog::new();
        catalog
            .insert(
                &tag("ru"),
                "upload",
                "{who} {actor, select, feminine{загрузила} other{загрузил}} \
                 {n, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}",
            )
            .expect("valid pattern");
        let localizer = Localizer::new(tag("ru")).with_provider(Box::new(catalog));

        let maria = MessageArgs::new()
            .with_text("who", "Мария")
            .with_gender("actor", &GrammaticalGender::feminine())
            .with_integer("n", 21);
        assert_eq!(
            localizer.format("upload", &maria),
            "Мария загрузила 21 файл"
        );

        let ivan = MessageArgs::new()
            .with_text("who", "Иван")
            .with_gender("actor", &GrammaticalGender::masculine())
            .with_integer("n", 111);
        assert_eq!(
            localizer.format("upload", &ivan),
            "Иван загрузил 111 файлов"
        );
    }

    #[test]
    fn debug_names_the_locale_without_a_debug_provider() {
        let localizer = Localizer::new(tag("ru-RU"));
        let text = format!("{localizer:?}");
        assert!(text.contains("ru"), "{text}");
    }
}
