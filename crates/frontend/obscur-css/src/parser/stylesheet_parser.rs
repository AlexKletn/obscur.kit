#![doc = include_str!("../../docs/api/parser/stylesheet_parser.doc/_module.md")]

mod declaration_terminator;
mod stylesheet_builder;

use super::ParsedStylesheet;
use crate::source::CssSource;
use stylesheet_builder::StylesheetBuilder;

#[doc = include_str!("../../docs/api/parser/stylesheet_parser.doc/StylesheetParser.md")]
#[derive(Debug)]
pub struct StylesheetParser<'a> {
    source: &'a CssSource,
}

impl<'a> StylesheetParser<'a> {
    /// Creates a parser for `source`.
    #[must_use]
    pub fn new(source: &'a CssSource) -> Self {
        Self { source }
    }

    /// Parses the source into a [`ParsedStylesheet`], recovering from errors.
    #[must_use]
    pub fn parse(&self) -> ParsedStylesheet {
        StylesheetBuilder::parse(self.source.text())
    }
}

#[cfg(test)]
mod tests {
    use super::StylesheetParser;
    use crate::error::UnsupportedFeatureKind;
    use crate::parser::{KeyframeSelector, ParsedRule, ParsedStylesheet};
    use crate::source::CssSource;

    /// The parsed sheet for a snippet of CSS.
    fn sheet_of(css: &str) -> ParsedStylesheet {
        StylesheetParser::new(&CssSource::anonymous(css)).parse()
    }

    #[test]
    fn a_stray_closing_brace_is_one_fault_and_not_an_endless_loop() {
        // The defect this caught, and it was not a hang alone: the recovery
        // path stopped in front of the `}` without stepping over it, so the
        // rule loop retried the same token forever and pushed a fresh error
        // every turn. A one-character typo in a stylesheet exhausted the
        // machine's memory.
        let sheet = sheet_of("a{color:red} } b{color:blue}");
        assert_eq!(sheet.errors.len(), 1, "one stray brace, one fault");
        assert_eq!(sheet.rules.len(), 2, "and the rules on both sides survived");
    }

    #[test]
    fn a_run_of_stray_braces_terminates() {
        let sheet = sheet_of("}}}}");
        assert_eq!(sheet.errors.len(), 4);
        assert!(sheet.rules.is_empty());
    }

    #[test]
    fn a_prelude_that_never_opens_a_block_terminates() {
        // The other way into the same loop: a selector with no `{` at all.
        let sheet = sheet_of("a b c");
        assert!(!sheet.errors.is_empty());
        assert!(sheet.rules.is_empty());
    }

    #[test]
    fn parses_simple_style_rule() {
        let src = CssSource::anonymous("a { color: red; }");
        let sheet = StylesheetParser::new(&src).parse();
        assert!(sheet.errors.is_empty());
        assert_eq!(sheet.rules.len(), 1);
        let ParsedRule::Style(rule) = &sheet.rules[0] else {
            panic!("expected style rule");
        };
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(&*rule.declarations[0].name, "color");
    }

    #[test]
    fn parses_multiple_declarations() {
        let src = CssSource::anonymous("a { color: red; width: 10px }");
        let sheet = StylesheetParser::new(&src).parse();
        let ParsedRule::Style(rule) = &sheet.rules[0] else {
            panic!("expected style rule");
        };
        assert_eq!(rule.declarations.len(), 2);
    }

    #[test]
    fn error_recovery_continues_after_bad_rule() {
        let src = CssSource::anonymous("a { color: red } b @@@ { } c { width: 1px }");
        let sheet = StylesheetParser::new(&src).parse();
        let style_rules = sheet
            .rules
            .iter()
            .filter(|r| matches!(r, ParsedRule::Style(_)))
            .count();
        assert!(style_rules >= 2, "expected at least 2 recovered rules");
        assert!(!sheet.errors.is_empty());
    }

    #[test]
    fn unsupported_at_rule_recorded() {
        let src = CssSource::anonymous("@import \"x.css\"; a { color: red }");
        let sheet = StylesheetParser::new(&src).parse();
        assert_eq!(sheet.unsupported.len(), 1);
        assert_eq!(sheet.unsupported[0].name(), "@import");
        assert!(
            sheet
                .rules
                .iter()
                .any(|r| matches!(r, ParsedRule::Style(_)))
        );
    }

    #[test]
    fn container_at_rule_is_reported_unsupported() {
        let src = CssSource::anonymous("@container (min-width: 400px) { a { color: red } }");
        let sheet = StylesheetParser::new(&src).parse();
        assert_eq!(sheet.unsupported.len(), 1);
        assert_eq!(sheet.unsupported[0].name(), "@container");
        assert_eq!(sheet.unsupported[0].kind(), UnsupportedFeatureKind::AtRule);
        // Reported, but still parsed: the rule is not dropped from the AST.
        assert!(
            sheet
                .rules
                .iter()
                .any(|r| matches!(r, ParsedRule::Container(_)))
        );
    }

    #[test]
    fn media_at_rule_is_not_reported_unsupported() {
        // `@media` really is wired; only `@container` is inert.
        let src = CssSource::anonymous("@media (min-width: 400px) { a { color: red } }");
        let sheet = StylesheetParser::new(&src).parse();
        assert!(sheet.unsupported.is_empty());
    }

    #[test]
    fn parses_nested_media_rule() {
        let src = CssSource::anonymous("@media screen { a { color: red } }");
        let sheet = StylesheetParser::new(&src).parse();
        let ParsedRule::Media(rule) = &sheet.rules[0] else {
            panic!("expected media rule");
        };
        assert_eq!(rule.rules.len(), 1);
    }

    #[test]
    fn parses_keyframes() {
        let src = CssSource::anonymous("@keyframes spin { from { opacity: 0 } to { opacity: 1 } }");
        let sheet = StylesheetParser::new(&src).parse();
        let ParsedRule::Keyframes(rule) = &sheet.rules[0] else {
            panic!("expected keyframes rule");
        };
        assert_eq!(&*rule.name, "spin");
        assert_eq!(rule.keyframes.len(), 2);
        assert_eq!(rule.keyframes[0].selectors, vec![KeyframeSelector::From]);
    }

    #[test]
    fn keyframes_percentage_selectors() {
        let src = CssSource::anonymous("@keyframes k { 0%, 50% { opacity: 1 } }");
        let sheet = StylesheetParser::new(&src).parse();
        let ParsedRule::Keyframes(rule) = &sheet.rules[0] else {
            panic!("expected keyframes rule");
        };
        assert_eq!(
            rule.keyframes[0].selectors,
            vec![
                KeyframeSelector::Percent(0.0),
                KeyframeSelector::Percent(50.0)
            ]
        );
    }

    #[test]
    fn empty_stylesheet() {
        let src = CssSource::anonymous("   \n  ");
        let sheet = StylesheetParser::new(&src).parse();
        assert!(sheet.rules.is_empty());
        assert!(sheet.errors.is_empty());
    }
}
