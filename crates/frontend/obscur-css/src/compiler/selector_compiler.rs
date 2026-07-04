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
use crate::parser::{ParsedComplexSelector, ParsedCompoundSelector, ParsedSelectorList, SimpleSelector};
use crate::source::SourceSpan;

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
}
