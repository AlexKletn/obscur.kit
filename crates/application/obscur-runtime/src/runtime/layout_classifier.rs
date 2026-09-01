#![doc = include_str!("layout_classifier.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_layout::invalidation::LayoutInvalidation;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::invalidation::StyleInvalidation;
use obscur_style::property::{PropertyId, PropertyValue, layout_keyword};

use super::{FramePlan, LayoutInputs, StyleDelta};
use crate::frame::{FrameContext, FrameReason};
use crate::invalidation::{Invalidation, InvalidationReason, InvalidationRouter};
use crate::orchestration::ComputedStyles;

#[doc = include_str!("layout_classifier.doc/LayoutPhase.md")]
pub(crate) enum LayoutPhase {
    #[doc = include_str!("layout_classifier.doc/Skip.md")]
    Skip,
    #[doc = include_str!("layout_classifier.doc/Full.md")]
    Full,
    #[doc = include_str!("layout_classifier.doc/Invalidated.md")]
    Invalidated(LayoutInvalidation),
}

#[doc = include_str!("layout_classifier.doc/LayoutClassifier.md")]
pub(crate) struct LayoutClassifier<'a> {
    #[doc = include_str!("layout_classifier.doc/last_inputs.md")]
    last_inputs: Option<&'a LayoutInputs>,
    #[doc = include_str!("layout_classifier.doc/current_inputs.md")]
    current_inputs: &'a LayoutInputs,
    #[doc = include_str!("layout_classifier.doc/retained.md")]
    retained: &'a ComputedStyles,
    #[doc = include_str!("layout_classifier.doc/overlaid.md")]
    overlaid: &'a ComputedStyles,
}

impl<'a> LayoutClassifier<'a> {
    #[doc = include_str!("layout_classifier.doc/new.md")]
    pub(crate) fn new(
        last_inputs: Option<&'a LayoutInputs>,
        current_inputs: &'a LayoutInputs,
        retained: &'a ComputedStyles,
        overlaid: &'a ComputedStyles,
    ) -> Self {
        Self {
            last_inputs,
            current_inputs,
            retained,
            overlaid,
        }
    }

    #[doc = include_str!("layout_classifier.doc/classify.md")]
    pub(crate) fn classify(
        &self,
        plan: FramePlan,
        ctx: &FrameContext,
        first_frame: bool,
        reconciled: bool,
        computed: &ComputedStyles,
        delta: &StyleDelta,
    ) -> LayoutPhase {
        if !plan.layout() {
            return LayoutPhase::Skip;
        }
        // The tree actually changed, or there is nothing retained to reuse.
        if reconciled || first_frame {
            return LayoutPhase::Full;
        }
        // Only the reasons whose layout inputs are fully covered below are refined.
        if !matches!(
            ctx.reason(),
            FrameReason::Input | FrameReason::Animation | FrameReason::UiCommit
        ) {
            return LayoutPhase::Full;
        }
        // Layout also reads the viewport, the scroll offset and the live editing
        // text — none of which are computed styles.
        let Some(last) = self.last_inputs else {
            return LayoutPhase::Full;
        };
        // Destructured without `..` on purpose: a new layout input added to
        // [`LayoutInputs`] breaks this line rather than being silently ignored
        // by a classifier that would then reuse boxes it never proved valid.
        let LayoutInputs {
            viewport,
            scroll,
            editing,
            language,
        } = self.current_inputs;
        if last.viewport != *viewport || last.editing != *editing || last.language != *language {
            return LayoutPhase::Full;
        }
        if last.scroll != *scroll && scroll_reaches_layout(computed) {
            return LayoutPhase::Full;
        }

        // Route the REAL per-node deltas: only a node whose layout-affecting
        // properties changed dirties the layout phase.
        let mut router = InvalidationRouter::new();
        let mut style_invalidation = StyleInvalidation::new();
        let mut layout_invalidation = LayoutInvalidation::new();
        let mut route = |node: NodeId, previous: Option<&ComputedStyle>| {
            if computed
                .get(&node)
                .is_some_and(|style| layout_affecting_changed(previous, style))
            {
                router.route(
                    Invalidation::new(Some(node), InvalidationReason::StyleChanged),
                    &mut style_invalidation,
                    &mut layout_invalidation,
                );
            }
        };
        match delta {
            StyleDelta::Full => {
                // A node cannot appear/disappear without a reconcile, but if the
                // styled set differs at all the retained boxes cannot be proven
                // valid.
                if computed.len() != self.retained.len() {
                    return LayoutPhase::Full;
                }
                for &node in computed.keys() {
                    route(node, self.retained.get(&node));
                }
            }
            StyleDelta::Incremental {
                replaced,
                introduced,
            } => {
                // A node the retained map was missing has no "before", so absence
                // from `replaced` proves nothing about it. Cannot happen while the
                // retained map covers the tree; if it ever does, lay out.
                if *introduced {
                    return LayoutPhase::Full;
                }
                // The candidates are every node this frame's style phase wrote:
                // the ones it re-resolved, plus the ones the motion overlay
                // sampled onto *after* it returned. A node in neither kept a
                // style nothing touched, so its box cannot have moved.
                //
                // For an overlay-only node the stash IS its previous style: it
                // held a pure cascade value entering the frame (or the restore in
                // `restyle_invalidated` would have listed it in `replaced`), and
                // nothing re-resolved it, so that value is what the last frame's
                // layout ran on.
                for &node in replaced.keys().chain(self.overlaid.keys()) {
                    route(
                        node,
                        replaced.get(&node).or_else(|| self.overlaid.get(&node)),
                    );
                }
            }
        }
        // Act on both halves of what the router just produced: its verdict decides
        // WHETHER the phase runs, and the accumulator it filled decides WHICH
        // contexts it re-measures. Reading only the boolean — and dropping the
        // per-node set — is what used to make `LayoutInvalidation` write-only and
        // layout all-or-nothing.
        if router.needs_layout() {
            LayoutPhase::Invalidated(layout_invalidation)
        } else {
            LayoutPhase::Skip
        }
    }
}

#[doc = include_str!("layout_classifier.doc/scroll_reaches_layout.md")]
fn scroll_reaches_layout(computed: &ComputedStyles) -> bool {
    computed.values().any(|style| {
        matches!(
            style.get(PropertyId::Position).map(ComputedValue::value),
            Some(&PropertyValue::Keyword(sym)) if sym == layout_keyword::POSITION_STICKY
        )
    })
}

#[doc = include_str!("layout_classifier.doc/layout_affecting_changed.md")]
fn layout_affecting_changed(old: Option<&ComputedStyle>, new: &ComputedStyle) -> bool {
    let Some(old) = old else {
        return true;
    };
    new.iter()
        .any(|(id, value)| id.affects_layout() && old.get(*id) != Some(value))
        || old
            .iter()
            .any(|(id, value)| id.affects_layout() && new.get(*id) != Some(value))
}

#[cfg(test)]
mod tests {
    use obscur_geometry::Size;

    use super::*;
    use crate::frame::FrameId;

    fn inputs() -> LayoutInputs {
        LayoutInputs {
            viewport: Size::new(100.0, 100.0),
            scroll: (0.0, 0.0),
            editing: None,
            language: "und".into(),
        }
    }

    fn context(reason: FrameReason) -> FrameContext {
        FrameContext::new(FrameId::FIRST, reason, Size::new(100.0, 100.0))
    }

    #[test]
    fn a_plan_that_already_skips_layout_is_left_alone() {
        // A compose-only text edit plans no layout at all, and the classifier only
        // ever removes work — so it must not put the phase back.
        let current = inputs();
        let styles = ComputedStyles::default();
        let classifier = LayoutClassifier::new(Some(&current), &current, &styles, &styles);
        let plan = FramePlan::for_frame(FrameReason::TextEdit, false);
        assert!(!plan.layout());
        assert!(matches!(
            classifier.classify(
                plan,
                &context(FrameReason::TextEdit),
                false,
                false,
                &styles,
                &StyleDelta::Full,
            ),
            LayoutPhase::Skip
        ));
    }

    #[test]
    fn nothing_retained_to_compare_against_lays_out_in_full() {
        // The first frame has no retained inputs at all, so the classifier cannot
        // prove anything and must take the whole pass.
        let current = inputs();
        let styles = ComputedStyles::default();
        let classifier = LayoutClassifier::new(None, &current, &styles, &styles);
        let plan = FramePlan::for_frame(FrameReason::Input, true);
        assert!(matches!(
            classifier.classify(
                plan,
                &context(FrameReason::Input),
                true,
                false,
                &styles,
                &StyleDelta::Full,
            ),
            LayoutPhase::Full
        ));
    }

    #[test]
    fn a_moved_non_style_input_lays_out_in_full_however_still_the_styles_are() {
        let last = inputs();
        let mut current = inputs();
        current.viewport = Size::new(100.0, 140.0);
        let styles = ComputedStyles::default();
        let classifier = LayoutClassifier::new(Some(&last), &current, &styles, &styles);
        let plan = FramePlan::for_frame(FrameReason::Input, false);
        assert!(matches!(
            classifier.classify(
                plan,
                &context(FrameReason::Input),
                false,
                false,
                &styles,
                &StyleDelta::Full,
            ),
            LayoutPhase::Full
        ));
    }

    /// One styled node whose `position` is the given keyword.
    fn positioned(keyword: obscur_identity::InternedSymbol) -> ComputedStyles {
        let mut style = ComputedStyle::new();
        style.set(PropertyId::Position, ComputedValue::keyword(keyword));
        let mut styles = ComputedStyles::default();
        styles.insert(NodeId::from_raw(1), style);
        styles
    }

    /// Classifies a scroll-only [`FrameReason::Input`] frame over `styles`,
    /// with the styles unchanged since the last frame (so nothing but the
    /// scroll offset moved).
    fn classify_scroll_only(styles: &ComputedStyles) -> LayoutPhase {
        let last = inputs();
        let mut current = inputs();
        current.scroll = (0.0, 40.0);
        LayoutClassifier::new(Some(&last), &current, styles, styles).classify(
            FramePlan::for_frame(FrameReason::Input, false),
            &context(FrameReason::Input),
            false,
            false,
            styles,
            &StyleDelta::Full,
        )
    }

    #[test]
    fn scrolling_a_document_without_sticky_boxes_skips_layout_entirely() {
        // Every box is laid out in DOCUMENT coordinates; the scroll offset only
        // ever reaches `Inset::sticky_offset`. So with no sticky box the boxes
        // this frame would produce are byte-identical to the retained ones, and
        // re-deriving them would flush both layout caches (`Full` passes
        // `dirty = None`) and re-shape every visible text leaf — once per
        // scrolled pixel.
        assert!(matches!(
            classify_scroll_only(&positioned(layout_keyword::POSITION_RELATIVE)),
            LayoutPhase::Skip
        ));
        // The same holds for a document that declares no `position` at all.
        assert!(matches!(
            classify_scroll_only(&ComputedStyles::default()),
            LayoutPhase::Skip
        ));
    }

    #[test]
    fn scrolling_a_document_with_a_sticky_box_still_lays_out_in_full() {
        // A sticky box's used position IS a function of the scroll offset, so
        // this is the one case the skip above must not swallow.
        assert!(matches!(
            classify_scroll_only(&positioned(layout_keyword::POSITION_STICKY)),
            LayoutPhase::Full
        ));
    }
}
