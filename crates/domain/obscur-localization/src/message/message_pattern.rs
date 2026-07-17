//! `MessagePattern`: a parsed translatable message, and the selection it
//! performs (spec §7.2).

use crate::message::{MessageArgs, MessagePatternError, MessageValue};
use crate::plural::{PluralCategory, PluralRules};

/// A parsed message pattern: literal text, interpolations, and the arm selection
/// that makes plural, gender and case agreement possible.
///
/// # Syntax
///
/// A deliberate subset of ICU MessageFormat, spelled the same way so a pattern
/// here reads the way translators already expect:
///
/// ```text
/// Привет, {name}!
/// {count, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}
/// {actor, select, feminine{добавила} other{добавил}} файл
/// нет {noun, select, genitive{файла} other{файл}}
/// ```
///
/// - `{name}` interpolates the argument's text.
/// - `{name, plural, <arm>…}` selects on the argument's [`PluralCategory`] under
///   the *formatting locale's* [`PluralRules`]. Arms are CLDR category keywords
///   (`zero one two few many other`) or `=<integer>` exact matches, which win over
///   the category when they match (`=0{no files} other{# files}`). Inside a
///   `plural` arm, `#` interpolates the number.
/// - `{name, select, <arm>…}` selects by name — this is how *gender* and *case*
///   variants are expressed, since both are a choice among arms keyed by a value
///   the caller binds.
/// - Every selector must have an `other` arm; [`parse`](Self::parse) rejects one
///   that does not.
/// - Arms nest: an arm's body is a full pattern.
///
/// # No escape syntax
///
/// `{`, `}` and (inside a `plural` arm) `#` cannot appear as literal text. ICU
/// quotes them with apostrophes; this subset does not implement that, and rather
/// than accept an apostrophe convention it cannot honour, [`parse`](Self::parse)
/// **rejects** a stray brace. A message that must render a literal brace cannot
/// be expressed, and will say so at parse time rather than at paint time.
#[derive(Debug, Clone, PartialEq)]
pub struct MessagePattern {
    parts: Vec<PatternPart>,
}

/// One piece of a parsed pattern.
#[derive(Debug, Clone, PartialEq)]
enum PatternPart {
    /// Literal text, emitted verbatim.
    Literal(Box<str>),
    /// `{name}` — the argument's text.
    Argument(Box<str>),
    /// `#` inside a `plural` arm — the number being pluralised.
    Number,
    /// `{name, plural, …}`.
    Plural {
        /// The argument selected on.
        name: Box<str>,
        /// `=<integer>` arms, checked before the category arms.
        exact: Vec<(u64, MessagePattern)>,
        /// Category arms.
        categories: Vec<(PluralCategory, MessagePattern)>,
        /// The mandatory `other` arm.
        other: MessagePattern,
    },
    /// `{name, select, …}`.
    Select {
        /// The argument selected on.
        name: Box<str>,
        /// Named arms, in source order.
        arms: Vec<(Box<str>, MessagePattern)>,
        /// The mandatory `other` arm.
        other: MessagePattern,
    },
}

impl MessagePattern {
    /// Parses `source`; see the type docs for the syntax and
    /// [`MessagePatternError`] for what is refused.
    pub fn parse(source: &str) -> Result<Self, MessagePatternError> {
        let mut parser = Parser {
            input: source.as_bytes(),
            at: 0,
        };
        let pattern = parser.pattern(false)?;
        if parser.at != parser.input.len() {
            // Only an unmatched `}` stops the top-level scan early.
            return Err(MessagePatternError::UnexpectedBrace);
        }
        Ok(pattern)
    }

    /// Formats the pattern with `args`, selecting plural arms under `rules`.
    ///
    /// `rules` is the *formatting locale's*, which is the whole point: the same
    /// pattern text formats differently under `ru` and `en` because the category
    /// of 21 differs. A [`Localizer`](crate::catalog::Localizer) pairs the two;
    /// this method is what it calls.
    ///
    /// # Missing and mismatched arguments
    ///
    /// Formatting never fails and never panics — it is on the frame path, and a
    /// missing argument is a catalogue bug that must not take the UI down.
    /// Instead it renders visibly:
    ///
    /// - an unbound `{name}` renders as `{name}`, brace-and-all, so it is obvious
    ///   on screen and greppable in a screenshot;
    /// - a `plural` over an unbound or non-numeric argument takes the `other`
    ///   arm, which is the only arm guaranteed to exist;
    /// - `#` outside any number renders empty.
    #[must_use]
    pub fn format(&self, args: &MessageArgs, rules: &PluralRules) -> String {
        let mut out = String::new();
        self.write(&mut out, args, rules, None);
        out
    }

    /// Appends this pattern's rendering to `out`.
    ///
    /// `number` is the literal `#` stands for — `Some` only inside a `plural`
    /// arm, and inherited by nested patterns so `#` works at any depth.
    fn write(
        &self,
        out: &mut String,
        args: &MessageArgs,
        rules: &PluralRules,
        number: Option<&str>,
    ) {
        for part in &self.parts {
            match part {
                PatternPart::Literal(text) => out.push_str(text),
                PatternPart::Number => out.push_str(number.unwrap_or("")),
                PatternPart::Argument(name) => match args.get(name) {
                    Some(value) => out.push_str(value.literal()),
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
                    let literal = value.map(MessageValue::literal);
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
                    arm.write(out, args, rules, literal);
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
                    arm.write(out, args, rules, number);
                }
            }
        }
    }
}

/// A hand-written recursive-descent parser over the pattern bytes.
///
/// Byte-oriented: every token it looks for (`{`, `}`, `,`, `#`) is ASCII, and it
/// only ever slices at those positions, so multi-byte text passes through intact.
struct Parser<'a> {
    input: &'a [u8],
    at: usize,
}

impl<'a> Parser<'a> {
    /// Parses a pattern until end of input or an unmatched `}`.
    ///
    /// `in_plural_arm` enables `#`; outside one, a `#` is ordinary text (there is
    /// no number for it to mean, and rejecting it would outlaw `#1` in every
    /// message).
    fn pattern(&mut self, in_plural_arm: bool) -> Result<MessagePattern, MessagePatternError> {
        let mut parts = Vec::new();
        let mut literal = String::new();
        while self.at < self.input.len() {
            match self.input[self.at] {
                b'}' => break,
                b'{' => {
                    if !literal.is_empty() {
                        parts.push(PatternPart::Literal(std::mem::take(&mut literal).into()));
                    }
                    self.at += 1;
                    parts.push(self.placeholder(in_plural_arm)?);
                }
                b'#' if in_plural_arm => {
                    if !literal.is_empty() {
                        parts.push(PatternPart::Literal(std::mem::take(&mut literal).into()));
                    }
                    self.at += 1;
                    parts.push(PatternPart::Number);
                }
                _ => {
                    let start = self.at;
                    self.at += 1;
                    // Walk to the end of this UTF-8 sequence so the slice is valid.
                    while self.at < self.input.len() && self.input[self.at] & 0xC0 == 0x80 {
                        self.at += 1;
                    }
                    // Both ends land on char boundaries of the original `&str`.
                    literal.push_str(
                        std::str::from_utf8(&self.input[start..self.at])
                            .map_err(|_| MessagePatternError::UnexpectedBrace)?,
                    );
                }
            }
        }
        if !literal.is_empty() {
            parts.push(PatternPart::Literal(literal.into()));
        }
        Ok(MessagePattern { parts })
    }

    /// Parses the body of a `{…}` placeholder; `self.at` is just past the `{`.
    ///
    /// `in_plural_arm` says whether this placeholder sits inside a `plural` arm.
    /// It is carried in rather than recomputed because it must **survive** a
    /// nested `select`: in `{n, plural, other{{g, select, …{# штук}…}}}` the `#`
    /// is lexically inside the select's arm but semantically still the plural's
    /// number, and dropping the flag at the select would parse it as a literal
    /// `#`.
    fn placeholder(&mut self, in_plural_arm: bool) -> Result<PatternPart, MessagePatternError> {
        let name = self.token(|b| b == b',' || b == b'}')?;
        if name.is_empty() {
            return Err(MessagePatternError::EmptyArgumentName);
        }
        match self.peek() {
            Some(b'}') => {
                self.at += 1;
                Ok(PatternPart::Argument(name.into()))
            }
            Some(b',') => {
                self.at += 1;
                let selector = self.token(|b| b == b',' || b == b'}')?;
                if self.peek() != Some(b',') {
                    return Err(MessagePatternError::NoArms);
                }
                self.at += 1;
                match selector {
                    // A `plural`'s own arms enable `#` regardless of context.
                    "plural" => self.plural_arms(name),
                    // A `select`'s arms inherit it: see this function's docs.
                    "select" => self.select_arms(name, in_plural_arm),
                    other => Err(MessagePatternError::UnknownSelector(other.into())),
                }
            }
            _ => Err(MessagePatternError::UnclosedPlaceholder),
        }
    }

    /// Parses `<arm>…}` for a `plural` placeholder.
    fn plural_arms(&mut self, name: &str) -> Result<PatternPart, MessagePatternError> {
        let arms = self.arms(true)?;
        let mut exact = Vec::new();
        let mut categories = Vec::new();
        let mut other = None;
        for (arm, pattern) in arms {
            if let Some(digits) = arm.strip_prefix('=') {
                let value = digits
                    .parse::<u64>()
                    .map_err(|_| MessagePatternError::UnknownPluralArm(arm.clone()))?;
                exact.push((value, pattern));
            } else if &*arm == "other" {
                other = Some(pattern);
            } else if let Some(category) = PluralCategory::from_keyword(&arm) {
                categories.push((category, pattern));
            } else {
                return Err(MessagePatternError::UnknownPluralArm(arm));
            }
        }
        Ok(PatternPart::Plural {
            name: name.into(),
            exact,
            categories,
            other: other.ok_or(MessagePatternError::MissingOtherArm)?,
        })
    }

    /// Parses `<arm>…}` for a `select` placeholder, whose arms inherit
    /// `in_plural_arm` from the context the placeholder appears in.
    fn select_arms(
        &mut self,
        name: &str,
        in_plural_arm: bool,
    ) -> Result<PatternPart, MessagePatternError> {
        let mut arms = Vec::new();
        let mut other = None;
        for (arm, pattern) in self.arms(in_plural_arm)? {
            if &*arm == "other" {
                other = Some(pattern);
            } else {
                arms.push((arm, pattern));
            }
        }
        Ok(PatternPart::Select {
            name: name.into(),
            arms,
            other: other.ok_or(MessagePatternError::MissingOtherArm)?,
        })
    }

    /// Parses `name{body}` arms up to the placeholder's closing `}`.
    fn arms(
        &mut self,
        in_plural: bool,
    ) -> Result<Vec<(Box<str>, MessagePattern)>, MessagePatternError> {
        let mut arms = Vec::new();
        loop {
            self.skip_space();
            match self.peek() {
                Some(b'}') => {
                    self.at += 1;
                    break;
                }
                None => return Err(MessagePatternError::UnclosedPlaceholder),
                _ => {}
            }
            let arm = self.token(|b| b == b'{')?.to_owned();
            if arm.is_empty() {
                return Err(MessagePatternError::EmptyArgumentName);
            }
            if self.peek() != Some(b'{') {
                return Err(MessagePatternError::UnclosedPlaceholder);
            }
            self.at += 1;
            let body = self.pattern(in_plural)?;
            if self.peek() != Some(b'}') {
                return Err(MessagePatternError::UnclosedPlaceholder);
            }
            self.at += 1;
            arms.push((arm.into_boxed_str(), body));
        }
        if arms.is_empty() {
            return Err(MessagePatternError::NoArms);
        }
        Ok(arms)
    }

    /// Reads a trimmed token up to the first byte `stop` accepts, without
    /// consuming it.
    fn token(&mut self, stop: impl Fn(u8) -> bool) -> Result<&'a str, MessagePatternError> {
        let start = self.at;
        while self.at < self.input.len() && !stop(self.input[self.at]) {
            self.at += 1;
        }
        // The scan stops only on ASCII, so both ends are char boundaries.
        std::str::from_utf8(&self.input[start..self.at])
            .map(str::trim)
            .map_err(|_| MessagePatternError::UnexpectedBrace)
    }

    /// Skips ASCII whitespace between arms.
    fn skip_space(&mut self) {
        while self.at < self.input.len() && self.input[self.at].is_ascii_whitespace() {
            self.at += 1;
        }
    }

    /// Returns the byte at the cursor, if any.
    fn peek(&self) -> Option<u8> {
        self.input.get(self.at).copied()
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
