use obscur_style::matching::Stylesheet;
use obscur_style::selector::ComplexSelector;

use super::candidate_rule::CandidateRule;
use super::generated_content::{AFTER, BEFORE, PLACEHOLDER_PART};
use super::subject_part_name;

#[doc = include_str!("../../docs/api/orchestration/generated_content.doc/PseudoRuleIndex.md")]
pub(super) struct PseudoRuleIndex<'a> {
    /// Rules with a selector whose subject carries `::before`.
    pub(super) before: Vec<CandidateRule<'a>>,
    /// Rules with a selector whose subject carries `::after`.
    pub(super) after: Vec<CandidateRule<'a>>,
    /// Rules with a selector whose subject is the exported `placeholder` part.
    pub(super) placeholder: Vec<CandidateRule<'a>>,
}

impl<'a> PseudoRuleIndex<'a> {
    #[doc = include_str!("../../docs/api/orchestration/generated_content.doc/of.md")]
    pub(super) fn of(sheets: &'a [Stylesheet]) -> Self {
        let mut index = Self {
            before: Vec::new(),
            after: Vec::new(),
            placeholder: Vec::new(),
        };
        // The declaration counter runs over EVERY declaration of EVERY rule,
        // matched or not, because that number is what the cascade tie-breaks
        // document order on. Advancing it by whole blocks leaves a kept rule's
        // declarations on exactly the numbers the unfiltered walk gave them, so
        // dropping the rules that cannot match cannot reorder the ones that can.
        let mut order: u32 = 0;
        for sheet in sheets {
            for rule in sheet.rules() {
                let candidate = CandidateRule {
                    origin: sheet.origin(),
                    rule,
                    order,
                };
                order = order.wrapping_add(u32::try_from(rule.block().len()).unwrap_or(u32::MAX));
                let selectors = rule.selectors().selectors();
                if selectors.iter().any(|s| targets_pseudo_element(s, BEFORE)) {
                    index.before.push(candidate);
                }
                if selectors.iter().any(|s| targets_pseudo_element(s, AFTER)) {
                    index.after.push(candidate);
                }
                if selectors
                    .iter()
                    .any(|s| subject_part_name(s) == Some(PLACEHOLDER_PART))
                {
                    index.placeholder.push(candidate);
                }
            }
        }
        index
    }
}

#[doc = include_str!("../../docs/api/orchestration/generated_content.doc/targets_pseudo_element.md")]
fn targets_pseudo_element(selector: &ComplexSelector, pseudo: &str) -> bool {
    selector
        .subject()
        .simples()
        .iter()
        .any(|simple| simple.pseudo_element_name() == Some(pseudo))
}
