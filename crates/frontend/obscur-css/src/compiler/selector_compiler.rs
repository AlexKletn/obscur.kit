#![doc = include_str!("selector_compiler.doc/_module.md")]

use crate::error::{UnsupportedCssFeature, UnsupportedFeatureKind};
use crate::parser::{
    AttrOp, Combinator, ParsedComplexSelector, ParsedCompoundSelector, ParsedSelectorList,
    SelectorParser, SimpleSelector,
};
use crate::source::{CssSource, SourceSpan};

use obscur_style::selector as style;
use obscur_style::selector::{SelectorId, SelectorSpecificity};

#[doc = include_str!("selector_compiler.doc/CompiledSelector.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompiledSelector {
    #[doc = include_str!("selector_compiler.doc/id.md")]
    pub id: SelectorId,
    #[doc = include_str!("selector_compiler.doc/specificity.md")]
    pub specificity: SelectorSpecificity,
}

#[doc = include_str!("selector_compiler.doc/SelectorCompiler.md")]
#[derive(Debug, Clone, Default)]
pub struct SelectorCompiler {
    next_id: u64,
}

impl SelectorCompiler {
    #[doc = include_str!("selector_compiler.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self { next_id: 0 }
    }

    #[doc = include_str!("selector_compiler.doc/specificity_2.md")]
    #[must_use]
    pub fn specificity(selector: &ParsedComplexSelector) -> SelectorSpecificity {
        Self::lower_complex(selector).specificity()
    }

    #[doc = include_str!("selector_compiler.doc/compile.md")]
    #[must_use]
    pub fn compile(
        &mut self,
        selector: &ParsedComplexSelector,
        unsupported: &mut Vec<UnsupportedCssFeature>,
    ) -> CompiledSelector {
        let specificity = Self::specificity(selector);
        collect_unsupported(selector, unsupported);
        let id = SelectorId::from_raw(self.next_id);
        self.next_id += 1;
        CompiledSelector { id, specificity }
    }

    #[doc = include_str!("selector_compiler.doc/compile_list.md")]
    #[must_use]
    pub fn compile_list(
        &mut self,
        list: &ParsedSelectorList,
        unsupported: &mut Vec<UnsupportedCssFeature>,
    ) -> Vec<CompiledSelector> {
        list.0
            .iter()
            .map(|selector| self.compile(selector, unsupported))
            .collect()
    }

    #[doc = include_str!("selector_compiler.doc/lower_list.md")]
    #[must_use]
    pub fn lower_list(list: &ParsedSelectorList) -> style::SelectorList {
        style::SelectorList::new(list.0.iter().map(Self::lower_complex).collect())
    }

    #[doc = include_str!("selector_compiler.doc/lower_complex.md")]
    #[must_use]
    pub fn lower_complex(selector: &ParsedComplexSelector) -> style::ComplexSelector {
        let head = Self::lower_compound(&selector.head);
        let tail = selector
            .tail
            .iter()
            .map(|(combinator, compound)| {
                (
                    lower_combinator(*combinator),
                    Self::lower_compound(compound),
                )
            })
            .collect();
        style::ComplexSelector::new(head, tail)
    }

    #[doc = include_str!("selector_compiler.doc/lower_compound.md")]
    #[must_use]
    pub fn lower_compound(compound: &ParsedCompoundSelector) -> style::CompoundSelector {
        style::CompoundSelector::new(compound.parts.iter().map(lower_simple).collect())
    }
}

#[doc = include_str!("selector_compiler.doc/lower_combinator.md")]
const fn lower_combinator(combinator: Combinator) -> style::Combinator {
    match combinator {
        Combinator::Descendant => style::Combinator::Descendant,
        Combinator::Child => style::Combinator::Child,
        Combinator::NextSibling => style::Combinator::NextSibling,
        Combinator::SubsequentSibling => style::Combinator::SubsequentSibling,
    }
}

#[doc = include_str!("selector_compiler.doc/lower_simple.md")]
fn lower_simple(simple: &SimpleSelector) -> style::SimpleSelector {
    match simple {
        SimpleSelector::Type(name) => style::SimpleSelector::Type(name.clone()),
        SimpleSelector::Universal => style::SimpleSelector::Universal,
        SimpleSelector::Class(name) => style::SimpleSelector::Class(name.clone()),
        SimpleSelector::Id(name) => style::SimpleSelector::Id(name.clone()),
        SimpleSelector::PseudoElement(name) => style::SimpleSelector::PseudoElement(name.clone()),
        SimpleSelector::Part(name) => style::SimpleSelector::Part(name.clone()),
        SimpleSelector::Attribute { name, op, value } => {
            // A missing operator or operand is a bare presence test (`[a]`).
            let matcher = match (op, value.as_deref()) {
                (Some(op), Some(operand)) => Some(lower_attr_op(*op, operand)),
                _ => None,
            };
            style::SimpleSelector::Attribute(style::AttributeSelector {
                name: name.clone(),
                matcher,
            })
        }
        SimpleSelector::PseudoClass { name, arg } => {
            style::SimpleSelector::PseudoClass(lower_pseudo_class(name, arg.as_deref()))
        }
    }
}

#[doc = include_str!("selector_compiler.doc/lower_attr_op.md")]
fn lower_attr_op(op: AttrOp, operand: &str) -> style::AttrMatch {
    match op {
        AttrOp::Exact => style::AttrMatch::Exact(operand.into()),
        AttrOp::Includes => style::AttrMatch::Includes(operand.into()),
        AttrOp::DashMatch => style::AttrMatch::DashMatch(operand.into()),
        AttrOp::Prefix => style::AttrMatch::Prefix(operand.into()),
        AttrOp::Suffix => style::AttrMatch::Suffix(operand.into()),
        AttrOp::Substring => style::AttrMatch::Substring(operand.into()),
    }
}

#[doc = include_str!("selector_compiler.doc/lower_pseudo_class.md")]
fn lower_pseudo_class(name: &str, arg: Option<&str>) -> style::PseudoClassSelector {
    use style::PseudoClassSelector as P;
    let lowered_name = name.to_ascii_lowercase();
    match lowered_name.as_str() {
        // `:is()` and `:where()` take a `<forgiving-selector-list>`; `:not()` and
        // `:has()` take an ordinary one. selectors-4 §16.1 is explicit that the
        // forgiving production "is used in :is() and :where() only", so an
        // unparsable branch invalidates the whole of `:not()`/`:has()` rather than
        // being dropped from it.
        "is" => return P::Is(parse_forgiving_list(arg.unwrap_or_default())),
        "where" => return P::Where(parse_forgiving_list(arg.unwrap_or_default())),
        "not" => {
            return parse_strict_list(arg.unwrap_or_default())
                .map_or_else(|| P::Other(lowered_name.clone().into()), P::Not);
        }
        "has" => {
            return parse_strict_list(arg.unwrap_or_default())
                .map_or_else(|| P::Other(lowered_name.clone().into()), P::Has);
        }
        "nth-child" => {
            let (anb, of) = split_anb_of(arg.unwrap_or_default());
            return match of {
                Some(list) => P::NthChildOf(parse_anb(anb), parse_nested_list(list)),
                None => P::NthChild(parse_anb(anb)),
            };
        }
        "nth-last-child" if split_anb_of(arg.unwrap_or_default()).1.is_some() => {
            return P::Other(name.into());
        }
        _ => {}
    }
    match lowered_name.as_str() {
        "hover" => P::Hover,
        "focus" => P::Focus,
        "focus-visible" => P::FocusVisible,
        "focus-within" => P::FocusWithin,
        "active" => P::Active,
        "disabled" => P::Disabled,
        "enabled" => P::Enabled,
        "checked" => P::Checked,
        "first-child" => P::FirstChild,
        "last-child" => P::LastChild,
        "only-child" => P::OnlyChild,
        "root" => P::Root,
        "empty" => P::Empty,
        "nth-last-child" => P::NthLastChild(parse_anb(arg.unwrap_or_default())),
        "nth-of-type" => P::NthOfType(parse_anb(arg.unwrap_or_default())),
        "nth-last-of-type" => P::NthLastOfType(parse_anb(arg.unwrap_or_default())),
        _ => P::Other(name.into()),
    }
}

#[doc = include_str!("selector_compiler.doc/parse_nested_list.md")]
fn parse_nested_list(arg: &str) -> style::SelectorList {
    let trimmed = arg.trim();
    if trimmed.is_empty() {
        return style::SelectorList::default();
    }
    let source = CssSource::anonymous(trimmed);
    SelectorParser::new()
        .parse(&source)
        .map(|list| SelectorCompiler::lower_list(&list))
        .unwrap_or_default()
}

#[doc = include_str!("selector_compiler.doc/parse_forgiving_list.md")]
fn parse_forgiving_list(arg: &str) -> style::SelectorList {
    parse_nested_list(arg)
}

#[doc = include_str!("selector_compiler.doc/parse_strict_list.md")]
fn parse_strict_list(arg: &str) -> Option<style::SelectorList> {
    let trimmed = arg.trim();
    if trimmed.is_empty() {
        return None;
    }
    let source = CssSource::anonymous(trimmed);
    let parsed = SelectorParser::new().parse(&source).ok()?;
    let lowered = SelectorCompiler::lower_list(&parsed);
    // A list that lowered to nothing had no usable branch at all.
    (!lowered.selectors().is_empty()).then_some(lowered)
}

#[doc = include_str!("selector_compiler.doc/split_anb_of.md")]
fn split_anb_of(arg: &str) -> (&str, Option<&str>) {
    let bytes = arg.as_bytes();
    let mut depth: i32 = 0;
    let mut quote: Option<u8> = None;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' | b'\'' => quote = Some(b),
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b'o' | b'O'
                if depth == 0
                    && i + 1 < bytes.len()
                    && bytes[i + 1].eq_ignore_ascii_case(&b'f')
                    && !is_ident_byte(if i == 0 {
                        None
                    } else {
                        bytes.get(i - 1).copied()
                    })
                    && !is_ident_byte(bytes.get(i + 2).copied()) =>
            {
                return (&arg[..i], Some(&arg[i + 2..]));
            }
            _ => {}
        }
        i += 1;
    }
    (arg, None)
}

#[doc = include_str!("selector_compiler.doc/is_ident_byte.md")]
fn is_ident_byte(byte: Option<u8>) -> bool {
    byte.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

#[doc = include_str!("selector_compiler.doc/parse_anb.md")]
fn parse_anb(arg: &str) -> style::AnB {
    let compact: String = arg.chars().filter(|c| !c.is_whitespace()).collect();
    let lower = compact.to_ascii_lowercase();
    if lower == "odd" {
        return style::AnB::new(2, 1);
    }
    if lower == "even" {
        return style::AnB::new(2, 0);
    }
    if let Some(index) = lower.find('n') {
        let a = match &lower[..index] {
            "" | "+" => 1,
            "-" => -1,
            other => other.parse::<i32>().unwrap_or(0),
        };
        let rest = &lower[index + 1..];
        let b = if rest.is_empty() {
            0
        } else {
            rest.strip_prefix('+')
                .unwrap_or(rest)
                .parse::<i32>()
                .unwrap_or(0)
        };
        return style::AnB::new(a, b);
    }
    style::AnB::new(0, lower.parse::<i32>().unwrap_or(0))
}

#[doc = include_str!("selector_compiler.doc/collect_unsupported.md")]
fn collect_unsupported(
    selector: &ParsedComplexSelector,
    unsupported: &mut Vec<UnsupportedCssFeature>,
) {
    let compounds = std::iter::once(&selector.head).chain(selector.tail.iter().map(|(_, c)| c));
    for compound in compounds {
        for part in &compound.parts {
            match part {
                SimpleSelector::PseudoClass { name, arg }
                    if is_excluded_pseudo(name, arg.as_deref()) =>
                {
                    unsupported.push(UnsupportedCssFeature::new(
                        UnsupportedFeatureKind::Selector,
                        format!(":{name}"),
                        SourceSpan::empty(0),
                    ));
                }
                // No `content` property and no generated box: a pseudo-element
                // rule styles nothing at all rather than leaking onto the
                // originating element.
                SimpleSelector::PseudoElement(name) => {
                    unsupported.push(UnsupportedCssFeature::new(
                        UnsupportedFeatureKind::PseudoElement,
                        format!("::{name}"),
                        SourceSpan::empty(0),
                    ));
                }
                _ => {}
            }
        }
    }
}

#[doc = include_str!("selector_compiler.doc/is_excluded_pseudo.md")]
fn is_excluded_pseudo(name: &str, arg: Option<&str>) -> bool {
    name.eq_ignore_ascii_case("link")
        || name.eq_ignore_ascii_case("visited")
        || name.eq_ignore_ascii_case("target")
        || name.eq_ignore_ascii_case("has")
        || (name.eq_ignore_ascii_case("nth-last-child")
            && split_anb_of(arg.unwrap_or_default()).1.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SelectorParser;
    use crate::source::CssSource;

    fn selector(text: &str) -> ParsedComplexSelector {
        let src = CssSource::anonymous(text);
        SelectorParser::new()
            .parse(&src)
            .unwrap()
            .0
            .into_iter()
            .next()
            .unwrap()
    }

    fn specificity(text: &str) -> SelectorSpecificity {
        SelectorCompiler::specificity(&selector(text))
    }

    #[test]
    fn id_selector_specificity() {
        assert_eq!(specificity("#main"), SelectorSpecificity::new(1, 0, 0));
    }

    #[test]
    fn class_and_attribute_and_pseudo_class_specificity() {
        assert_eq!(specificity(".btn"), SelectorSpecificity::new(0, 1, 0));
        assert_eq!(
            specificity("[type=text]"),
            SelectorSpecificity::new(0, 1, 0)
        );
        assert_eq!(specificity(":hover"), SelectorSpecificity::new(0, 1, 0));
    }

    #[test]
    fn type_and_pseudo_element_specificity() {
        assert_eq!(specificity("div"), SelectorSpecificity::new(0, 0, 1));
        assert_eq!(specificity("::before"), SelectorSpecificity::new(0, 0, 1));
    }

    #[test]
    fn universal_selector_contributes_nothing() {
        assert_eq!(specificity("*"), SelectorSpecificity::ZERO);
    }

    #[test]
    fn compound_selector_sums_parts() {
        // div.a#b:hover -> 1 id, 1 class + 1 pseudo-class, 1 type.
        assert_eq!(
            specificity("div.a#b:hover"),
            SelectorSpecificity::new(1, 2, 1)
        );
    }

    #[test]
    fn complex_selector_sums_across_combinators() {
        // ul > li.item + a#link -> ids: 1, classes: 1, types: 3.
        assert_eq!(
            specificity("ul > li.item + a#link"),
            SelectorSpecificity::new(1, 1, 3)
        );
    }

    #[test]
    fn one_id_outranks_many_classes() {
        assert!(specificity("#a") > specificity(".a.b.c.d"));
    }

    #[test]
    fn compile_mints_sequential_ids() {
        let mut compiler = SelectorCompiler::new();
        let mut unsupported = Vec::new();
        let first = compiler.compile(&selector("a"), &mut unsupported);
        let second = compiler.compile(&selector("b"), &mut unsupported);
        assert_eq!(first.id, SelectorId::from_raw(0));
        assert_eq!(second.id, SelectorId::from_raw(1));
        assert!(unsupported.is_empty());
    }

    #[test]
    fn compile_list_mints_one_id_per_selector() {
        let src = CssSource::anonymous("a, b, c");
        let list = SelectorParser::new().parse(&src).unwrap();
        let mut compiler = SelectorCompiler::new();
        let mut unsupported = Vec::new();
        let compiled = compiler.compile_list(&list, &mut unsupported);
        assert_eq!(compiled.len(), 3);
        assert_eq!(compiled[2].id, SelectorId::from_raw(2));
    }

    #[test]
    fn visited_pseudo_class_is_unsupported() {
        let mut compiler = SelectorCompiler::new();
        let mut unsupported = Vec::new();
        let _ = compiler.compile(&selector("a:visited"), &mut unsupported);
        assert_eq!(unsupported.len(), 1);
        assert_eq!(unsupported[0].kind(), UnsupportedFeatureKind::Selector);
        assert_eq!(unsupported[0].name(), ":visited");
    }

    #[test]
    fn pseudo_element_is_unsupported() {
        let mut compiler = SelectorCompiler::new();
        let mut unsupported = Vec::new();
        let _ = compiler.compile(&selector(".badge::before"), &mut unsupported);
        let _ = compiler.compile(&selector("li::after"), &mut unsupported);
        assert_eq!(unsupported.len(), 2);
        assert_eq!(unsupported[0].kind(), UnsupportedFeatureKind::PseudoElement);
        assert_eq!(unsupported[0].name(), "::before");
        assert_eq!(unsupported[1].name(), "::after");
    }

    #[test]
    fn pseudo_element_in_a_tail_compound_is_reported() {
        let mut compiler = SelectorCompiler::new();
        let mut unsupported = Vec::new();
        let _ = compiler.compile(&selector("ul > li::before"), &mut unsupported);
        assert_eq!(unsupported.len(), 1);
        assert_eq!(unsupported[0].name(), "::before");
    }

    #[test]
    fn link_and_target_are_unsupported() {
        let mut compiler = SelectorCompiler::new();
        let mut unsupported = Vec::new();
        let _ = compiler.compile(&selector("a:link"), &mut unsupported);
        let _ = compiler.compile(&selector("section:target"), &mut unsupported);
        assert_eq!(unsupported.len(), 2);
    }

    #[test]
    fn hover_is_supported() {
        let mut compiler = SelectorCompiler::new();
        let mut unsupported = Vec::new();
        let _ = compiler.compile(&selector("a:hover"), &mut unsupported);
        assert!(unsupported.is_empty());
    }

    #[test]
    fn excluded_pseudo_in_combinator_tail_is_detected() {
        let mut compiler = SelectorCompiler::new();
        let mut unsupported = Vec::new();
        let _ = compiler.compile(&selector("nav > a:visited"), &mut unsupported);
        assert_eq!(unsupported.len(), 1);
    }

    /// Lowers `text` (a single complex selector) and returns its subject
    /// compound's simple selectors.
    fn lower_subject_simples(text: &str) -> Vec<style::SimpleSelector> {
        SelectorCompiler::lower_complex(&selector(text))
            .subject()
            .simples()
            .to_vec()
    }

    #[test]
    fn lowers_type_universal_class_id() {
        assert_eq!(
            lower_subject_simples("div"),
            vec![style::SimpleSelector::Type("div".into())]
        );
        assert_eq!(
            lower_subject_simples("*"),
            vec![style::SimpleSelector::Universal]
        );
        assert_eq!(
            lower_subject_simples(".a"),
            vec![style::SimpleSelector::Class("a".into())]
        );
        assert_eq!(
            lower_subject_simples("#b"),
            vec![style::SimpleSelector::Id("b".into())]
        );
    }

    #[test]
    fn lowers_attribute_presence() {
        assert_eq!(
            lower_subject_simples("[data-x]"),
            vec![style::SimpleSelector::Attribute(style::AttributeSelector {
                name: "data-x".into(),
                matcher: None,
            })]
        );
    }

    #[test]
    fn lowers_every_attribute_operator() {
        let cases = [
            ("[type=\"text\"]", style::AttrMatch::Exact("text".into())),
            ("[a~=\"x\"]", style::AttrMatch::Includes("x".into())),
            ("[a|=\"x\"]", style::AttrMatch::DashMatch("x".into())),
            ("[a^=\"x\"]", style::AttrMatch::Prefix("x".into())),
            ("[a$=\"x\"]", style::AttrMatch::Suffix("x".into())),
            ("[a*=\"x\"]", style::AttrMatch::Substring("x".into())),
        ];
        for (text, expected) in cases {
            let simples = lower_subject_simples(text);
            let style::SimpleSelector::Attribute(attr) = &simples[0] else {
                panic!("expected attribute selector for {text}");
            };
            assert_eq!(attr.matcher, Some(expected), "for {text}");
        }
    }

    #[test]
    fn lowers_state_pseudo_class() {
        assert_eq!(
            lower_subject_simples(":hover"),
            vec![style::SimpleSelector::PseudoClass(
                style::PseudoClassSelector::Hover
            )]
        );
    }

    #[test]
    fn lowers_nth_child_and_keywords() {
        let anb = |text: &str| {
            let simples = lower_subject_simples(text);
            match &simples[0] {
                style::SimpleSelector::PseudoClass(style::PseudoClassSelector::NthChild(anb)) => {
                    *anb
                }
                other => panic!("expected nth-child, got {other:?}"),
            }
        };
        assert_eq!(anb(":nth-child(2n+1)"), style::AnB::new(2, 1));
        assert_eq!(anb(":nth-child(odd)"), style::AnB::new(2, 1));
        assert_eq!(anb(":nth-child(even)"), style::AnB::new(2, 0));
        assert_eq!(anb(":nth-child(3)"), style::AnB::new(0, 3));
    }

    #[test]
    fn unknown_pseudo_class_lowers_to_other() {
        assert_eq!(
            lower_subject_simples(":unknown"),
            vec![style::SimpleSelector::PseudoClass(
                style::PseudoClassSelector::Other("unknown".into())
            )]
        );
    }

    #[test]
    fn lowers_combinators() {
        assert_eq!(
            SelectorCompiler::lower_complex(&selector("a b")).tail()[0].0,
            style::Combinator::Descendant
        );
        assert_eq!(
            SelectorCompiler::lower_complex(&selector("a > b")).tail()[0].0,
            style::Combinator::Child
        );
        assert_eq!(
            SelectorCompiler::lower_complex(&selector("a + b")).tail()[0].0,
            style::Combinator::NextSibling
        );
        assert_eq!(
            SelectorCompiler::lower_complex(&selector("a ~ b")).tail()[0].0,
            style::Combinator::SubsequentSibling
        );
    }

    #[test]
    fn lower_list_preserves_order_and_count() {
        let src = CssSource::anonymous(".a, #b, div");
        let list = SelectorParser::new().parse(&src).unwrap();
        let lowered = SelectorCompiler::lower_list(&list);
        assert_eq!(lowered.selectors().len(), 3);
        assert_eq!(
            lowered.selectors()[0].subject().simples(),
            &[style::SimpleSelector::Class("a".into())]
        );
    }

    // ---- selector-list pseudo-classes --------------------------------------

    /// selectors-4 §15: ":is(em, #foo) has a specificity of (1,0,0)"; and
    /// ":not(em, strong#foo)" is (1,0,1).
    #[test]
    fn is_and_not_take_their_most_specific_argument() {
        assert_eq!(
            specificity(":is(em, #foo)"),
            SelectorSpecificity::new(1, 0, 0)
        );
        assert_eq!(
            specificity(":not(em, strong#foo)"),
            SelectorSpecificity::new(1, 0, 1)
        );
        // The replacement is per-pseudo-class, so the rest of the compound still
        // counts: `.qux:is(#foo)` is (1,1,0).
        assert_eq!(
            specificity(".qux:is(#foo)"),
            SelectorSpecificity::new(1, 1, 0)
        );
    }

    /// selectors-4 §15: ".qux:where(em, #foo#bar#baz) has a specificity of
    /// (0,1,0): only the .qux outside the :where() contributes."
    #[test]
    fn where_contributes_zero_specificity() {
        assert_eq!(
            specificity(".qux:where(em, #foo#bar#baz)"),
            SelectorSpecificity::new(0, 1, 0)
        );
    }

    /// selectors-4 §13.3.1: "S:nth-child(An+B) and :nth-child(An+B of S) have
    /// the exact same specificity".
    #[test]
    fn nth_child_of_s_ties_with_the_compound_form() {
        assert_eq!(
            specificity(":nth-child(-n+3 of li.important)"),
            specificity("li.important:nth-child(-n+3)")
        );
        assert_eq!(
            specificity(":nth-child(-n+3 of li.important)"),
            SelectorSpecificity::new(0, 2, 1)
        );
    }

    #[test]
    fn lowers_is_where_not_and_has_to_their_selector_lists() {
        use style::PseudoClassSelector as P;
        let arg = |text: &str| match &lower_subject_simples(text)[0] {
            style::SimpleSelector::PseudoClass(
                P::Is(list) | P::Where(list) | P::Not(list) | P::Has(list),
            ) => list.clone(),
            other => panic!("expected a selector-list pseudo-class, got {other:?}"),
        };
        for text in [
            ":is(.a, #b)",
            ":where(.a, #b)",
            ":not(.a, #b)",
            ":has(.a, #b)",
        ] {
            let list = arg(text);
            assert_eq!(list.selectors().len(), 2, "for {text}");
            assert_eq!(
                list.selectors()[0].subject().simples(),
                &[style::SimpleSelector::Class("a".into())],
                "for {text}"
            );
        }
    }

    #[test]
    fn lowers_nth_child_of_s_and_keeps_the_bare_form_allocation_free() {
        use style::PseudoClassSelector as P;
        match &lower_subject_simples(":nth-child(-n+3 of .li.important)")[0] {
            style::SimpleSelector::PseudoClass(P::NthChildOf(anb, list)) => {
                assert_eq!(*anb, style::AnB::new(-1, 3));
                assert_eq!(list.selectors().len(), 1);
            }
            other => panic!("expected nth-child-of, got {other:?}"),
        }
        assert!(matches!(
            &lower_subject_simples(":nth-child(-n+3)")[0],
            style::SimpleSelector::PseudoClass(P::NthChild(_))
        ));
    }

    /// A malformed or empty argument list matches nothing and scores zero,
    /// rather than being silently treated as "everything".
    #[test]
    fn unparseable_or_empty_argument_yields_an_empty_list() {
        use style::PseudoClassSelector as P;
        for text in [":is()", ":is(> )"] {
            match &lower_subject_simples(text)[0] {
                style::SimpleSelector::PseudoClass(P::Is(list)) => {
                    assert!(list.is_empty(), "for {text}");
                }
                other => panic!("expected :is, got {other:?}"),
            }
            assert_eq!(specificity(text), SelectorSpecificity::ZERO, "for {text}");
        }
    }

    #[test]
    fn has_and_nth_last_child_of_s_are_reported_unsupported() {
        let mut compiler = SelectorCompiler::new();
        let mut unsupported = Vec::new();
        let _ = compiler.compile(&selector(".card:has(.badge)"), &mut unsupported);
        let _ = compiler.compile(&selector(":nth-last-child(-n+3 of .li)"), &mut unsupported);
        assert_eq!(unsupported.len(), 2);
        assert_eq!(unsupported[0].name(), ":has");
        assert_eq!(unsupported[1].name(), ":nth-last-child");
        // The plain form stays supported and modelled.
        let mut plain = Vec::new();
        let _ = compiler.compile(&selector(":nth-last-child(2)"), &mut plain);
        assert!(plain.is_empty());
        assert!(matches!(
            &lower_subject_simples(":nth-last-child(2)")[0],
            style::SimpleSelector::PseudoClass(style::PseudoClassSelector::NthLastChild(_))
        ));
    }

    #[test]
    fn split_anb_of_finds_only_the_standalone_keyword() {
        assert_eq!(split_anb_of("2n+1"), ("2n+1", None));
        assert_eq!(
            split_anb_of("-n+3 of .li.important"),
            ("-n+3 ", Some(" .li.important"))
        );
        // `of` inside a class name is not the separator.
        assert_eq!(split_anb_of("2 of .of-item"), ("2 ", Some(" .of-item")));
        assert_eq!(split_anb_of("2n"), ("2n", None));
        // Nor is one nested inside brackets or a string.
        assert_eq!(split_anb_of("2n+1 [data-of]"), ("2n+1 [data-of]", None));
        assert_eq!(split_anb_of("2n+1 [x=\"of\"]"), ("2n+1 [x=\"of\"]", None));
    }

    #[test]
    fn parse_anb_grammar() {
        assert_eq!(parse_anb("2n+1"), style::AnB::new(2, 1));
        assert_eq!(parse_anb("odd"), style::AnB::new(2, 1));
        assert_eq!(parse_anb("even"), style::AnB::new(2, 0));
        assert_eq!(parse_anb("3"), style::AnB::new(0, 3));
        assert_eq!(parse_anb("-n+3"), style::AnB::new(-1, 3));
        assert_eq!(parse_anb("n"), style::AnB::new(1, 0));
        assert_eq!(parse_anb("2n - 1"), style::AnB::new(2, -1));
    }
}
