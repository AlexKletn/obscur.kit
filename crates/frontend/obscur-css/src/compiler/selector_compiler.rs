//! Computing selector specificity and minting selector ids (§4.13/§4.14).
//!
//! [`SelectorCompiler`] turns a parsed selector into a [`CompiledSelector`]: the
//! CSS `(a, b, c)` specificity triple the cascade ranks declarations by, plus a
//! sequential [`SelectorId`] handle that ties a produced [`Declaration`] back to
//! the rule that authored it. It also surfaces the excluded dynamic-state
//! pseudo-classes (`:link`, `:visited`, `:target`) as
//! [`UnsupportedCssFeature`]s (§4.13) rather than silently matching them.
//!
//! [`Declaration`]: obscur_style::cascade::Declaration

use crate::error::{UnsupportedCssFeature, UnsupportedFeatureKind};
use crate::parser::{
    AttrOp, Combinator, ParsedComplexSelector, ParsedCompoundSelector, ParsedSelectorList,
    SimpleSelector,
};
use crate::source::SourceSpan;

use obscur_style::selector as style;
use obscur_style::selector::{SelectorId, SelectorSpecificity};

/// A compiled selector: its cascade [`SelectorId`] and its specificity.
///
/// The parsed selector AST carries no per-token spans, so the excluded
/// pseudo-classes it triggers are reported against an empty span.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompiledSelector {
    /// The sequential handle minted for this selector.
    pub id: SelectorId,
    /// The computed CSS `(a, b, c)` specificity.
    pub specificity: SelectorSpecificity,
}

/// Computes specificity for parsed selectors and mints their [`SelectorId`]s.
///
/// Ids are handed out sequentially (starting at zero) across the lifetime of the
/// compiler, so every selector it compiles gets a distinct handle.
#[derive(Debug, Clone, Default)]
pub struct SelectorCompiler {
    next_id: u64,
}

impl SelectorCompiler {
    /// Creates a selector compiler whose first minted id is zero.
    #[must_use]
    pub const fn new() -> Self {
        Self { next_id: 0 }
    }

    /// Computes the CSS `(a, b, c)` specificity of a complex selector (§4.14).
    ///
    /// `a` counts id selectors, `b` counts class/attribute/pseudo-class
    /// selectors, and `c` counts type/pseudo-element selectors; the universal
    /// selector contributes nothing. Components are summed (saturating) across
    /// every compound in the complex selector.
    #[must_use]
    pub fn specificity(selector: &ParsedComplexSelector) -> SelectorSpecificity {
        let mut total = compound_specificity(&selector.head);
        for (_, compound) in &selector.tail {
            total = total.add(compound_specificity(compound));
        }
        total
    }

    /// Compiles one complex selector, minting its id and reporting any excluded
    /// pseudo-classes into `unsupported`.
    #[must_use]
    pub fn compile(
        &mut self,
        selector: &ParsedComplexSelector,
        unsupported: &mut Vec<UnsupportedCssFeature>,
    ) -> CompiledSelector {
        let specificity = Self::specificity(selector);
        collect_excluded(selector, unsupported);
        let id = SelectorId::from_raw(self.next_id);
        self.next_id += 1;
        CompiledSelector { id, specificity }
    }

    /// Compiles every complex selector in a selector list, in source order.
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

    /// Lowers a parsed selector list into the `obscur-style` selector model the
    /// runtime matcher consumes (§4.13).
    ///
    /// This is a pure, id-free structural translation: the parsed AST maps 1:1
    /// onto [`style::SelectorList`], preserving source order. Excluded
    /// dynamic-state pseudo-classes are *not* filtered here (that diagnostic is
    /// raised by [`compile`](Self::compile)); they lower to
    /// [`style::PseudoClassSelector::Other`], which the matcher treats as
    /// matching nothing.
    #[must_use]
    pub fn lower_list(list: &ParsedSelectorList) -> style::SelectorList {
        style::SelectorList::new(list.0.iter().map(Self::lower_complex).collect())
    }

    /// Lowers one parsed complex selector, mapping the head and each
    /// combinator-linked tail compound left-to-right.
    #[must_use]
    pub fn lower_complex(selector: &ParsedComplexSelector) -> style::ComplexSelector {
        let head = Self::lower_compound(&selector.head);
        let tail = selector
            .tail
            .iter()
            .map(|(combinator, compound)| {
                (lower_combinator(*combinator), Self::lower_compound(compound))
            })
            .collect();
        style::ComplexSelector::new(head, tail)
    }

    /// Lowers one parsed compound selector into its run of simple selectors.
    #[must_use]
    pub fn lower_compound(compound: &ParsedCompoundSelector) -> style::CompoundSelector {
        style::CompoundSelector::new(compound.parts.iter().map(lower_simple).collect())
    }
}

/// Maps a parser combinator onto its `obscur-style` counterpart (1:1 by name).
const fn lower_combinator(combinator: Combinator) -> style::Combinator {
    match combinator {
        Combinator::Descendant => style::Combinator::Descendant,
        Combinator::Child => style::Combinator::Child,
        Combinator::NextSibling => style::Combinator::NextSibling,
        Combinator::SubsequentSibling => style::Combinator::SubsequentSibling,
    }
}

/// Lowers one parsed simple selector into the `obscur-style` model.
///
/// Type/universal/class/id/pseudo-element map directly; attribute and
/// pseudo-class selectors delegate to [`lower_attr_op`]/[`lower_pseudo_class`].
fn lower_simple(simple: &SimpleSelector) -> style::SimpleSelector {
    match simple {
        SimpleSelector::Type(name) => style::SimpleSelector::Type(name.clone()),
        SimpleSelector::Universal => style::SimpleSelector::Universal,
        SimpleSelector::Class(name) => style::SimpleSelector::Class(name.clone()),
        SimpleSelector::Id(name) => style::SimpleSelector::Id(name.clone()),
        SimpleSelector::PseudoElement(name) => style::SimpleSelector::PseudoElement(name.clone()),
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

/// Pairs an attribute operator with its operand to build an [`style::AttrMatch`].
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

/// Lowers a pseudo-class name (and optional functional argument) into the
/// `obscur-style` [`style::PseudoClassSelector`].
///
/// The name is matched case-insensitively. Functional `:nth-*` forms parse their
/// argument via [`parse_anb`]; any name the model does not know is preserved as
/// [`style::PseudoClassSelector::Other`] (which matches nothing).
fn lower_pseudo_class(name: &str, arg: Option<&str>) -> style::PseudoClassSelector {
    use style::PseudoClassSelector as P;
    match name.to_ascii_lowercase().as_str() {
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
        "nth-child" => P::NthChild(parse_anb(arg.unwrap_or_default())),
        "nth-last-child" => P::NthLastChild(parse_anb(arg.unwrap_or_default())),
        "nth-of-type" => P::NthOfType(parse_anb(arg.unwrap_or_default())),
        "nth-last-of-type" => P::NthLastOfType(parse_anb(arg.unwrap_or_default())),
        _ => P::Other(name.into()),
    }
}

/// Parses an `An+B` micro-syntax argument into an [`style::AnB`].
///
/// Handles the keywords `odd` (`2n+1`) and `even` (`2n`), a bare integer `k`
/// (`0n+k`), and the general `An±B` grammar including the omitted-coefficient
/// forms `n` (`1n+0`) and `-n+B` (`-1n+B`). Whitespace inside the argument is
/// ignored. Anything unparseable degrades to `0n+0`, which matches no index.
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
            rest.strip_prefix('+').unwrap_or(rest).parse::<i32>().unwrap_or(0)
        };
        return style::AnB::new(a, b);
    }
    style::AnB::new(0, lower.parse::<i32>().unwrap_or(0))
}

/// Sums the specificity contributions of one compound selector's parts.
fn compound_specificity(compound: &ParsedCompoundSelector) -> SelectorSpecificity {
    let mut a: u16 = 0;
    let mut b: u16 = 0;
    let mut c: u16 = 0;
    for part in &compound.parts {
        match part {
            SimpleSelector::Id(_) => a = a.saturating_add(1),
            SimpleSelector::Class(_)
            | SimpleSelector::Attribute { .. }
            | SimpleSelector::PseudoClass { .. } => b = b.saturating_add(1),
            SimpleSelector::Type(_) | SimpleSelector::PseudoElement(_) => c = c.saturating_add(1),
            SimpleSelector::Universal => {}
        }
    }
    SelectorSpecificity::new(a, b, c)
}

/// Records the excluded dynamic-state pseudo-classes (`:link`, `:visited`,
/// `:target`) found anywhere in the complex selector (§4.13).
fn collect_excluded(selector: &ParsedComplexSelector, unsupported: &mut Vec<UnsupportedCssFeature>) {
    let compounds = std::iter::once(&selector.head).chain(selector.tail.iter().map(|(_, c)| c));
    for compound in compounds {
        for part in &compound.parts {
            if let SimpleSelector::PseudoClass { name, .. } = part
                && is_excluded_pseudo(name)
            {
                unsupported.push(UnsupportedCssFeature::new(
                    UnsupportedFeatureKind::Selector,
                    format!(":{name}"),
                    SourceSpan::empty(0),
                ));
            }
        }
    }
}

/// Returns whether a pseudo-class name is one the kit excludes (§4.13).
fn is_excluded_pseudo(name: &str) -> bool {
    name.eq_ignore_ascii_case("link")
        || name.eq_ignore_ascii_case("visited")
        || name.eq_ignore_ascii_case("target")
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
        assert_eq!(specificity("[type=text]"), SelectorSpecificity::new(0, 1, 0));
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
        assert_eq!(specificity("div.a#b:hover"), SelectorSpecificity::new(1, 2, 1));
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
        assert_eq!(lower_subject_simples("*"), vec![style::SimpleSelector::Universal]);
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
                style::SimpleSelector::PseudoClass(style::PseudoClassSelector::NthChild(anb)) => *anb,
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
