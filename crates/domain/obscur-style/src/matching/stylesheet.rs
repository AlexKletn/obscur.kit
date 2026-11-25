//! One cascade origin's worth of authored style rules: [`Stylesheet`] (§4.14).

use super::stylesheet_data::StylesheetData;
use crate::cascade::CascadeOrigin;
use crate::matching::StyleRule;
use crate::selector::{
    Combinator, CompoundSelector, MatchElement, PseudoClassSelector, SelectorList, SimpleSelector,
};
use obscur_identity::VersionStamp;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Mints one [`VersionStamp`] per sheet ever built in this process.
///
/// Never reused and never zero, so "same stamp" can only mean "descended by
/// cloning from the same construction". Wrapping would need a sheet built
/// every nanosecond for six hundred years.
static NEXT_STAMP: AtomicU64 = AtomicU64::new(1);

#[doc = include_str!("../../docs/api/matching/stylesheet.doc/Stylesheet.md")]
#[derive(Debug, Clone)]
pub struct Stylesheet {
    origin: CascadeOrigin,
    data: Arc<StylesheetData>,
    broad_child_structure: bool,
    container_style_queries: bool,
    #[doc = include_str!("../../docs/api/matching/stylesheet.doc/stamp.md")]
    stamp: VersionStamp,
}

#[doc = include_str!("../../docs/api/matching/stylesheet.doc/PartialEq.md")]
impl PartialEq for Stylesheet {
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin
            && (Arc::ptr_eq(&self.data, &other.data) || self.data.rules == other.data.rules)
    }
}

impl Stylesheet {
    /// Creates a stylesheet for `origin` from its rules in document order.
    #[must_use]
    pub fn new(origin: CascadeOrigin, rules: Vec<StyleRule>) -> Self {
        let mut data = StylesheetData {
            rules,
            sibling_targets: Vec::new(),
            container_properties: Vec::new(),
            container_ranges: Vec::new(),
            container_references: Vec::new(),
            container_ranges_use_ch: false,
        };
        let mut broad_child_structure = false;
        let mut container_style_queries = false;
        for rule in &data.rules {
            if let Some(condition) = rule.condition() {
                container_style_queries |= condition.uses_container_style();
                condition.collect_container_properties(
                    &mut data.container_properties,
                    &mut data.container_ranges,
                    &mut data.container_references,
                    &mut data.container_ranges_use_ch,
                );
            }
            broad_child_structure |=
                collect_child_dependencies(rule.selectors(), &mut data.sibling_targets);
        }
        Self {
            origin,
            data: Arc::new(data),
            broad_child_structure,
            container_style_queries,
            stamp: VersionStamp::from_raw(NEXT_STAMP.fetch_add(1, Ordering::Relaxed)),
        }
    }

    #[doc = include_str!("../../docs/api/matching/stylesheet.doc/stamp_2.md")]
    #[must_use]
    pub const fn stamp(&self) -> VersionStamp {
        self.stamp
    }

    /// Returns the cascade origin every rule in this sheet belongs to.
    #[must_use]
    pub const fn origin(&self) -> CascadeOrigin {
        self.origin
    }

    /// Returns the sheet's rules in document order.
    #[must_use]
    pub fn rules(&self) -> &[StyleRule] {
        &self.data.rules
    }

    /// Whether a rule can change its matches when a parent’s list of children changes.
    /// Includes rules inside conditions so a true result is a conservative signal.
    #[must_use]
    pub fn child_structure_sensitive(&self) -> bool {
        self.broad_child_structure || !self.data.sibling_targets.is_empty()
    }

    /// Whether positional or relational pseudo-classes require conservative child-list invalidation.
    #[must_use]
    pub const fn broad_child_structure(&self) -> bool {
        self.broad_child_structure
    }

    /// Whether the element could match the compound on the right of a sibling combinator. Uses mandatory class or ID keys, with a wildcard for unkeyed compounds.
    #[must_use]
    pub fn is_sibling_target(&self, element: &impl MatchElement) -> bool {
        self.data
            .sibling_targets
            .iter()
            .any(|selector| match selector {
                SimpleSelector::Id(id) => element.id() == Some(id),
                SimpleSelector::Class(class) => element.has_class(class),
                SimpleSelector::Universal => true,
                _ => unreachable!("sibling dependency keys"),
            })
    }

    /// Whether any rule needs style containers to be snapshotted.
    #[must_use]
    pub const fn uses_container_style_queries(&self) -> bool {
        self.container_style_queries
    }

    /// The computed longhands requested by this sheet’s container style queries.
    #[must_use]
    pub fn container_query_properties(&self) -> &[crate::property::PropertyId] {
        &self.data.container_properties
    }

    /// The custom properties compared by numeric style ranges in this sheet.
    #[must_use]
    pub fn container_query_ranges(&self) -> &[obscur_identity::InternedSymbol] {
        &self.data.container_ranges
    }

    /// The standard property and custom reference pairs used by this sheet’s style queries.
    #[must_use]
    pub fn container_query_references(
        &self,
    ) -> &[(crate::property::PropertyId, obscur_identity::InternedSymbol)] {
        &self.data.container_references
    }

    /// Whether a style query uses an expected value or fallback in ch units.
    #[must_use]
    pub fn container_ranges_use_ch(&self) -> bool {
        self.data.container_ranges_use_ch
    }

    /// Appends a rule to the end of the sheet (later in document order).
    pub fn push(&mut self, rule: StyleRule) {
        let data = Arc::make_mut(&mut self.data);
        self.broad_child_structure |=
            collect_child_dependencies(rule.selectors(), &mut data.sibling_targets);
        if let Some(condition) = rule.condition() {
            self.container_style_queries |= condition.uses_container_style();
            condition.collect_container_properties(
                &mut data.container_properties,
                &mut data.container_ranges,
                &mut data.container_references,
                &mut data.container_ranges_use_ch,
            );
        }
        data.rules.push(rule);
        // The stamp answers "are these the same rules?", so a sheet that grew a
        // rule is no longer the sheet it was, and a holder of the old stamp must
        // be told so.
        self.stamp = VersionStamp::from_raw(NEXT_STAMP.fetch_add(1, Ordering::Relaxed));
    }
}

fn collect_child_dependencies(list: &SelectorList, targets: &mut Vec<SimpleSelector>) -> bool {
    fn collect_compound(compound: &CompoundSelector, targets: &mut Vec<SimpleSelector>) -> bool {
        use PseudoClassSelector::*;
        compound.simples().iter().any(|simple| match simple {
            SimpleSelector::PseudoClass(pseudo) => match pseudo {
                Is(list) | Where(list) | Not(list) => collect_child_dependencies(list, targets),
                FirstChild | LastChild | OnlyChild | Empty | NthChild(_) | NthLastChild(_)
                | NthOfType(_) | NthLastOfType(_) | NthChildOf(..) | Has(_) => true,
                // The matcher treats an unknown pseudo-class as always false.
                Other(_) | Hover | Focus | FocusVisible | FocusWithin | Active | Disabled
                | Enabled | Checked | Root => false,
            },
            _ => false,
        })
    }

    for selector in list.selectors() {
        if collect_compound(selector.head(), targets) {
            return true;
        }
        for (combinator, compound) in selector.tail() {
            if collect_compound(compound, targets) {
                return true;
            }
            if matches!(
                combinator,
                Combinator::NextSibling | Combinator::SubsequentSibling
            ) {
                let key = compound
                    .simples()
                    .iter()
                    .find(|simple| {
                        matches!(simple, SimpleSelector::Id(_) | SimpleSelector::Class(_))
                    })
                    .cloned()
                    .unwrap_or(SimpleSelector::Universal);
                if !targets.contains(&key) {
                    targets.push(key);
                }
            }
        }
    }
    false
}

impl Default for Stylesheet {
    /// An empty stylesheet in the [`CascadeOrigin::GlobalApp`] origin.
    fn default() -> Self {
        Self::new(CascadeOrigin::GlobalApp, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matching::BlockDeclaration;
    use crate::property::{PropertyId, PropertyValue};
    use crate::selector::{ComplexSelector, CompoundSelector, SelectorList, SimpleSelector};

    fn rule() -> StyleRule {
        StyleRule::new(
            SelectorList::new(vec![ComplexSelector::new(
                CompoundSelector::new(vec![SimpleSelector::Type("div".into())]),
                Vec::new(),
            )]),
            vec![BlockDeclaration::normal(
                PropertyId::Opacity,
                PropertyValue::Number(1.0),
            )],
        )
    }

    #[test]
    fn editing_a_clone_keeps_old_rules_and_dependencies() {
        let original = Stylesheet::new(CascadeOrigin::GlobalApp, vec![rule()]);
        let mut edited = original.clone();
        assert!(Arc::ptr_eq(&original.data, &edited.data));
        let selector = ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::PseudoClass(
                PseudoClassSelector::Empty,
            )]),
            Vec::new(),
        );
        edited.push(StyleRule::new(
            SelectorList::new(vec![selector]),
            Vec::new(),
        ));
        assert_eq!(original.rules().len(), 1);
        assert_eq!(edited.rules().len(), 2);
        assert!(!original.broad_child_structure());
        assert!(edited.broad_child_structure());
        assert_ne!(original.stamp(), edited.stamp());
    }

    #[test]
    fn default_is_empty_global_app() {
        let sheet = Stylesheet::default();
        assert_eq!(sheet.origin(), CascadeOrigin::GlobalApp);
        assert!(sheet.rules().is_empty());
    }

    #[test]
    fn new_and_accessors() {
        let sheet = Stylesheet::new(CascadeOrigin::Theme, vec![rule()]);
        assert_eq!(sheet.origin(), CascadeOrigin::Theme);
        assert_eq!(sheet.rules().len(), 1);
    }

    #[test]
    fn push_appends_rules() {
        let mut sheet = Stylesheet::new(CascadeOrigin::Inline, Vec::new());
        sheet.push(rule());
        sheet.push(rule());
        assert_eq!(sheet.rules().len(), 2);
    }
}
