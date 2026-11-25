//! `MessagePattern`: a parsed translatable message, and the selection it
//! performs (spec §7.2).

use crate::message::{MessageArgs, MessagePatternError, MessageValue};
use crate::plural::PluralRules;

use super::{message_pattern_parser::MessagePatternParser, pattern_part::PatternPart};

#[doc = include_str!("../../docs/api/message/message_pattern.doc/MessagePattern.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct MessagePattern {
    parts: Vec<PatternPart>,
}

impl MessagePattern {
    pub(super) fn from_parts(parts: Vec<PatternPart>) -> Self {
        Self { parts }
    }

    /// Parses `source`; see the type docs for the syntax and
    /// [`MessagePatternError`] for what is refused.
    pub fn parse(source: &str) -> Result<Self, MessagePatternError> {
        MessagePatternParser::parse(source)
    }

    #[doc = include_str!("../../docs/api/message/message_pattern.doc/format.md")]
    #[must_use]
    pub fn format(&self, args: &MessageArgs, rules: &PluralRules) -> String {
        self.format_with(args, rules, |out, value| out.push_str(value.literal()))
    }

    pub(crate) fn format_with(
        &self,
        args: &MessageArgs,
        rules: &PluralRules,
        write_value: impl Fn(&mut String, &MessageValue),
    ) -> String {
        let mut out = String::new();
        self.write(&mut out, args, rules, None, &write_value);
        out
    }

    #[doc = include_str!("../../docs/api/message/message_pattern.doc/write.md")]
    fn write(
        &self,
        out: &mut String,
        args: &MessageArgs,
        rules: &PluralRules,
        number: Option<&MessageValue>,
        write_value: &impl Fn(&mut String, &MessageValue),
    ) {
        for part in &self.parts {
            match part {
                PatternPart::Literal(text) => out.push_str(text),
                PatternPart::Number => {
                    if let Some(value) = number {
                        write_value(out, value);
                    }
                }
                PatternPart::Argument(name) => match args.get(name) {
                    Some(value) => write_value(out, value),
                    // Visibly wrong beats invisibly missing.
                    None => {
                        out.push('{');
                        out.push_str(name);
                        out.push('}');
                    }
                },
                PatternPart::Plural {
                    name,
                    exact,
                    categories,
                    other,
                } => {
                    let value = args.get(name);
                    let operands = value.and_then(MessageValue::operands);
                    let arm = operands.map_or(other, |operands| {
                        // An `=<n>` arm wins over the category arm, per ICU: it is
                        // the more specific statement.
                        if operands.v == 0
                            && let Some((_, pattern)) = exact.iter().find(|(n, _)| *n == operands.i)
                        {
                            return pattern;
                        }
                        let category = rules.select(operands);
                        categories
                            .iter()
                            .find(|(c, _)| *c == category)
                            .map_or(other, |(_, pattern)| pattern)
                    });
                    arm.write(out, args, rules, value, write_value);
                }
                PatternPart::Select { name, arms, other } => {
                    let arm = args
                        .get(name)
                        .and_then(|value| {
                            let selector = value.selector_name();
                            arms.iter()
                                .find(|(n, _)| &**n == selector)
                                .map(|(_, pattern)| pattern)
                        })
                        .unwrap_or(other);
                    arm.write(out, args, rules, number, write_value);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::LanguageTag;
    use crate::message::{GrammaticalCase, GrammaticalGender};

    fn rules(tag: &str) -> PluralRules {
        PluralRules::cardinal(&LanguageTag::parse(tag).expect("tag"))
    }

    fn pattern(source: &str) -> MessagePattern {
        MessagePattern::parse(source).expect("valid pattern")
    }

    #[test]
    fn literal_text_passes_through() {
        let p = pattern("Привет!");
        assert_eq!(p.format(&MessageArgs::new(), &rules("ru")), "Привет!");
    }

    #[test]
    fn interpolates_a_named_argument() {
        let p = pattern("Привет, {name}!");
        let args = MessageArgs::new().with_text("name", "Мария");
        assert_eq!(p.format(&args, &rules("ru")), "Привет, Мария!");
    }

    #[test]
    fn an_unbound_argument_renders_visibly() {
        let p = pattern("Привет, {name}!");
        assert_eq!(
            p.format(&MessageArgs::new(), &rules("ru")),
            "Привет, {name}!"
        );
    }

    #[test]
    fn plural_selects_the_arm_and_hash_is_the_number() {
        let p = pattern("{count, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}");
        let ru = rules("ru");
        for (count, expected) in [(1, "1 файл"), (2, "2 файла"), (5, "5 файлов")] {
            let args = MessageArgs::new().with_integer("count", count);
            assert_eq!(p.format(&args, &ru), expected);
        }
    }

    #[test]
    fn the_same_pattern_formats_differently_per_locale() {
        // Not the pattern's doing: `rules` is the locale's, and 21 has different
        // categories in Russian and Polish.
        let p = pattern("{n, plural, one{one} few{few} many{many} other{other}}");
        let args = MessageArgs::new().with_integer("n", 21);
        assert_eq!(p.format(&args, &rules("ru")), "one");
        assert_eq!(p.format(&args, &rules("pl")), "many");
        assert_eq!(p.format(&args, &rules("en")), "other");
    }

    #[test]
    fn an_exact_arm_wins_over_the_category_arm() {
        let p = pattern("{n, plural, =0{нет файлов} one{# файл} many{# файлов} other{# файла}}");
        let ru = rules("ru");
        // 0 is `many` in Russian; the `=0` arm is more specific and takes it.
        assert_eq!(
            p.format(&MessageArgs::new().with_integer("n", 0), &ru),
            "нет файлов"
        );
        // ...and 5, also `many`, still takes the category arm.
        assert_eq!(
            p.format(&MessageArgs::new().with_integer("n", 5), &ru),
            "5 файлов"
        );
    }

    #[test]
    fn an_exact_arm_does_not_match_a_written_fraction() {
        // `=1` is an integer match: `1.0` is not it (`v != 0`).
        let p = pattern("{n, plural, =1{exact} other{other}}");
        let args = MessageArgs::new().with("n", MessageValue::decimal("1.0").expect("valid"));
        assert_eq!(p.format(&args, &rules("ru")), "other");
    }

    #[test]
    fn plural_over_a_missing_or_non_numeric_argument_takes_other() {
        let p = pattern("{n, plural, one{one} other{other}}");
        let en = rules("en");
        assert_eq!(p.format(&MessageArgs::new(), &en), "other");
        let args = MessageArgs::new().with_text("n", "1");
        assert_eq!(p.format(&args, &en), "other");
    }

    #[test]
    fn a_missing_category_arm_falls_back_to_other() {
        // A Russian message a translator only gave two arms: `few` is absent, so
        // 2 lands on `other` rather than panicking or vanishing.
        let p = pattern("{n, plural, one{# файл} other{# файла}}");
        let ru = rules("ru");
        assert_eq!(
            p.format(&MessageArgs::new().with_integer("n", 2), &ru),
            "2 файла"
        );
    }

    #[test]
    fn select_expresses_gender_agreement() {
        let p = pattern("{actor, select, feminine{добавила} other{добавил}} файл");
        let ru = rules("ru");
        let feminine = MessageArgs::new().with_gender("actor", &GrammaticalGender::feminine());
        let masculine = MessageArgs::new().with_gender("actor", &GrammaticalGender::masculine());
        assert_eq!(p.format(&feminine, &ru), "добавила файл");
        assert_eq!(p.format(&masculine, &ru), "добавил файл");
        // An unbound selector takes `other`.
        assert_eq!(p.format(&MessageArgs::new(), &ru), "добавил файл");
    }

    #[test]
    fn select_expresses_case_variants() {
        let p = pattern("{form, select, genitive{файла} prepositional{файле} other{файл}}");
        let ru = rules("ru");
        for (case, expected) in [
            (GrammaticalCase::genitive(), "файла"),
            (GrammaticalCase::prepositional(), "файле"),
            (GrammaticalCase::nominative(), "файл"),
        ] {
            let args = MessageArgs::new().with_case("form", &case);
            assert_eq!(p.format(&args, &ru), expected, "{case}");
        }
    }

    #[test]
    fn selectors_nest_and_hash_survives_the_nesting() {
        // Numeral agreement *and* gender in one message: the shape a real
        // "N files added by X" string needs.
        let p = pattern(
            "{actor, select, feminine{добавила} other{добавил}} \
             {count, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}",
        );
        let args = MessageArgs::new()
            .with_gender("actor", &GrammaticalGender::feminine())
            .with_integer("count", 22);
        assert_eq!(p.format(&args, &rules("ru")), "добавила 22 файла");
    }

    #[test]
    fn hash_inside_a_nested_select_still_means_the_number() {
        let p = pattern("{n, plural, other{{g, select, feminine{# штука} other{# штук}}}}");
        let args = MessageArgs::new()
            .with_integer("n", 7)
            .with_gender("g", &GrammaticalGender::feminine());
        assert_eq!(p.format(&args, &rules("ru")), "7 штука");
    }

    #[test]
    fn hash_outside_a_plural_arm_is_ordinary_text() {
        let p = pattern("issue #{n}");
        let args = MessageArgs::new().with_integer("n", 42);
        assert_eq!(p.format(&args, &rules("en")), "issue #42");
    }

    #[test]
    fn whitespace_between_arms_is_not_output() {
        let p = pattern("{n, plural,\n  one {# файл}\n  other {# файла}\n}");
        let args = MessageArgs::new().with_integer("n", 1);
        assert_eq!(p.format(&args, &rules("ru")), "1 файл");
    }

    #[test]
    fn rejects_malformed_patterns_rather_than_painting_them() {
        use MessagePatternError::{
            EmptyArgumentName, MissingOtherArm, NoArms, UnclosedPlaceholder, UnexpectedBrace,
            UnknownPluralArm, UnknownSelector,
        };
        let cases: &[(&str, MessagePatternError)] = &[
            ("{name", UnclosedPlaceholder),
            ("hi}", UnexpectedBrace),
            ("{}", EmptyArgumentName),
            ("{n, plural}", NoArms),
            ("{n, plural, }", NoArms),
            (
                "{n, ordinal, one{a} other{b}}",
                UnknownSelector("ordinal".into()),
            ),
            (
                "{n, plural, singular{a} other{b}}",
                UnknownPluralArm("singular".into()),
            ),
            ("{n, plural, one{a}}", MissingOtherArm),
            ("{g, select, feminine{a}}", MissingOtherArm),
            ("{n, plural, =x{a} other{b}}", UnknownPluralArm("=x".into())),
        ];
        for (source, expected) in cases {
            assert_eq!(
                MessagePattern::parse(source).unwrap_err(),
                *expected,
                "parsing {source}"
            );
        }
    }

    #[test]
    fn a_literal_brace_is_refused_at_parse_time_not_painted() {
        // The documented limitation, asserted so the doc cannot drift from it.
        assert!(MessagePattern::parse("a } b").is_err());
        assert!(MessagePattern::parse("{n, plural, one{a} other{b}} }").is_err());
    }

    #[test]
    fn multibyte_text_slices_on_char_boundaries() {
        // Every literal byte-walk must land on a boundary; Cyrillic is 2 bytes per
        // char and the arm bodies here are pure Cyrillic.
        let p = pattern("Всего {n, plural, one{# файл} many{# файлов} other{# файла}} — готово");
        let args = MessageArgs::new().with_integer("n", 100);
        assert_eq!(p.format(&args, &rules("ru")), "Всего 100 файлов — готово");
    }
}
