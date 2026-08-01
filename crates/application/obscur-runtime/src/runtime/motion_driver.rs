#![doc = include_str!("motion_driver.doc/_module.md")]

use std::collections::{HashMap, HashSet};

use obscur_identity::NodeId;
use obscur_motion::scheduler::MotionScheduler;
use obscur_motion::timeline::{Timeline, TimelineId};
use obscur_motion::track::{Keyframe, KeyframeTrack};
use obscur_motion::transition::TransitionInterruption;
use obscur_motion::value::MotionValue;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{Length, PropertyId, PropertyValue, Transform};
use obscur_style::selector::SelectorMatcher;
use obscur_tree::tree::RetainedTree;

use crate::orchestration::{
    AnimatedKind, AnimationBinding, ComputedStyles, CssMotionRules, CssTransitionRule,
    MotionCoordinator, MotionPlayback, PseudoStates, TreeElement,
};

#[doc = include_str!("motion_driver.doc/MotionDriver.md")]
#[derive(Debug)]
pub(crate) struct MotionDriver {
    #[doc = include_str!("motion_driver.doc/motion.md")]
    motion: MotionScheduler,
    #[doc = include_str!("motion_driver.doc/animations.md")]
    animations: Vec<AnimationBinding>,
    #[doc = include_str!("motion_driver.doc/presentation_overlays.md")]
    presentation_overlays: Vec<(NodeId, PropertyId, PropertyValue)>,
    #[doc = include_str!("motion_driver.doc/next_timeline.md")]
    next_timeline: TimelineId,
    #[doc = include_str!("motion_driver.doc/settled_progress.md")]
    settled_progress: HashMap<TimelineId, f32>,
    #[doc = include_str!("motion_driver.doc/css_motion.md")]
    css_motion: CssMotionRules,
    #[doc = include_str!("motion_driver.doc/transition_targets.md")]
    transition_targets: HashMap<(NodeId, PropertyId), MotionValue>,
    #[doc = include_str!("motion_driver.doc/transition_runs.md")]
    transition_runs: HashMap<(NodeId, PropertyId), TransitionRun>,
    #[doc = include_str!("motion_driver.doc/css_animations_started.md")]
    css_animations_started: HashSet<(NodeId, PropertyId)>,
}

impl MotionDriver {
    #[doc = include_str!("motion_driver.doc/new.md")]
    pub(crate) fn new() -> Self {
        Self {
            motion: MotionScheduler::new(),
            animations: Vec::new(),
            presentation_overlays: Vec::new(),
            next_timeline: TimelineId::from_raw(0),
            settled_progress: HashMap::new(),
            css_motion: CssMotionRules::new(),
            transition_targets: HashMap::new(),
            transition_runs: HashMap::new(),
            css_animations_started: HashSet::new(),
        }
    }

    #[doc = include_str!("motion_driver.doc/advance.md")]
    pub(crate) fn advance(&mut self, delta: f32) -> bool {
        for finished in self.motion.advance(delta) {
            self.settled_progress
                .insert(finished.id(), finished.progress());
        }
        self.motion.needs_frame()
    }

    #[doc = include_str!("motion_driver.doc/set_css_rules.md")]
    pub(crate) fn set_css_rules(&mut self, rules: CssMotionRules) {
        self.css_motion = rules;
        self.transition_targets.clear();
        self.transition_runs.clear();
        self.css_animations_started.clear();
    }

    #[doc = include_str!("motion_driver.doc/active_count.md")]
    pub(crate) fn active_count(&self) -> usize {
        self.motion.active_count()
    }

    #[doc = include_str!("motion_driver.doc/binding_count.md")]
    #[cfg(test)]
    pub(crate) fn binding_count(&self) -> usize {
        self.animations.len()
    }

    #[doc = include_str!("motion_driver.doc/animate_with.md")]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn animate_with(
        &mut self,
        node: NodeId,
        property: PropertyId,
        track: KeyframeTrack,
        duration: f32,
        kind: AnimatedKind,
        interruption: TransitionInterruption,
        playback: MotionPlayback,
    ) -> TimelineId {
        // Resolve the track to actually run against any animation already in
        // flight for this exact target (spec §4.17 interruption).
        let existing = self
            .animations
            .iter()
            .position(|binding| binding.node() == node && binding.property() == property);

        let resolved_track = if let Some(index) = existing {
            let prior = &self.animations[index];
            // The current on-screen value: the prior track sampled at its
            // timeline's progress (a finished/pruned timeline holds the progress
            // it settled at).
            let progress = self.timeline_progress(prior.timeline());
            let current = MotionCoordinator::sample_keyframes(prior.track(), progress);
            let origin = MotionCoordinator::sample_keyframes(prior.track(), 0.0);
            match (current, origin) {
                // Apply the interruption policy from the prior track's sampled
                // values. `None` means `Ignore` — keep the in-flight animation.
                (Some(current), Some(origin)) => {
                    match interruption.resolve_track(&track, current, origin) {
                        Some(resolved) => resolved,
                        None => return prior.timeline(),
                    }
                }
                // A degenerate (empty) prior track cannot be sampled; fall back to
                // the incoming track verbatim.
                _ => track,
            }
        } else {
            track
        };

        // Supersede any existing animation of the same target: pull its timeline
        // out of the scheduler, then drop its binding.
        while let Some(index) = self
            .animations
            .iter()
            .position(|binding| binding.node() == node && binding.property() == property)
        {
            let stale = self.animations.remove(index);
            self.motion.remove(stale.timeline());
            self.settled_progress.remove(&stale.timeline());
        }

        let id = self.next_timeline;
        self.next_timeline = TimelineId::from_raw(id.raw().saturating_add(1));

        let mut timeline = Timeline::new(id, duration)
            .with_delay(playback.delay_secs)
            .with_repeat(playback.repeat)
            .with_direction(playback.direction)
            .alternating(playback.alternates);
        // `start` arms a positive delay and applies a negative one, so a timeline
        // with `-0.5s` of delay is already 0.5s in before the first advance.
        timeline.start();
        // A timeline that a negative delay ran straight past its end never enters
        // the scheduler, so record where it settled before registering.
        if timeline.is_finished() {
            self.settled_progress.insert(id, timeline.progress());
        }
        self.motion.register(timeline);
        self.animations.push(
            AnimationBinding::new(id, node, property, resolved_track, kind)
                .with_fill(playback.fill),
        );

        id
    }

    #[doc = include_str!("motion_driver.doc/timeline_progress.md")]
    fn timeline_progress(&self, id: TimelineId) -> f32 {
        self.motion.active().get(id).map_or_else(
            || self.settled_progress.get(&id).copied().unwrap_or(1.0),
            Timeline::progress,
        )
    }

    #[doc = include_str!("motion_driver.doc/active_animations.md")]
    pub(crate) fn active_animations(&self) -> usize {
        self.active_count()
    }

    #[doc = include_str!("motion_driver.doc/apply_overlay.md")]
    pub(crate) fn apply_overlay(
        &mut self,
        computed: &mut ComputedStyles,
        overlaid: &mut ComputedStyles,
    ) {
        // Rebuilt from scratch each frame: the group-compositing presentation
        // values (`opacity`/`transform`) never touch computed style, so they
        // cannot be stashed-and-restored like the computed overlay — they are
        // re-derived from the live timelines every frame.
        let mut presentation: Vec<(NodeId, PropertyId, PropertyValue)> = Vec::new();
        for binding in &self.animations {
            // Inside the start delay the animation is not yet running: it applies
            // its first-iteration start value only under a `backwards`/`both`
            // fill, and nothing at all otherwise (css-animations-1 §3.9).
            if self
                .motion
                .active()
                .get(binding.timeline())
                .is_some_and(Timeline::is_delayed)
                && !binding.fill().fills_backwards()
            {
                continue;
            }
            // Read progress from the scheduler; a finished (pruned) timeline holds
            // the progress it settled at.
            let progress = self.timeline_progress(binding.timeline());

            let Some(sampled) = MotionCoordinator::sample_keyframes(binding.track(), progress)
            else {
                continue;
            };

            // `opacity` and `transform` are the two group-compositing properties:
            // motion applies them as *presentation* values that the scene composer
            // layers on at paint (a `PushLayer(opacity)` / `PushTransform`), and
            // they must never be written back into computed style (spec §4.17;
            // NG-MOTION-01). They also never affect layout, so a `transform`
            // animation moves paint without a relayout. The presentation lowering
            // is keyed off the PROPERTY, not the binding's `AnimatedKind`, so a
            // transform track presents as a matrix even when the caller labelled
            // its kind `Number`.
            if is_presentation_property(binding.property()) {
                if let Some(value) = lower_presentation(binding.property(), sampled) {
                    presentation.push((binding.node(), binding.property(), value));
                }
                continue;
            }

            let Some(property_value) = binding.kind().lower(sampled) else {
                continue;
            };
            let Some(value) = ComputedValue::from_concrete(property_value) else {
                continue;
            };
            if let Some(style) = computed.get_mut(&binding.node()) {
                // Stash the settled cascade value the first time this node is
                // overlaid since the last restyle, so the style phase can put it
                // back — see the `overlaid` field. `or_insert_with` is what makes
                // it the *first*: on a later frame `style` already carries a sample,
                // and snapshotting that would lose the cascade for good.
                overlaid
                    .entry(binding.node())
                    .or_insert_with(|| style.clone());
                style.set(binding.property(), value);
            }
        }
        self.presentation_overlays = presentation;
    }

    #[doc = include_str!("motion_driver.doc/presentation_styles.md")]
    pub(crate) fn presentation_styles(&self, computed: &ComputedStyles) -> Option<ComputedStyles> {
        if self.presentation_overlays.is_empty() {
            return None;
        }
        let mut styles = computed.clone();
        for (node, property, value) in &self.presentation_overlays {
            if let Some(computed_value) = ComputedValue::from_concrete(value.clone()) {
                styles
                    .entry(*node)
                    .or_default()
                    .set(*property, computed_value);
            }
        }
        Some(styles)
    }

    #[doc = include_str!("motion_driver.doc/drive_css.md")]
    pub(crate) fn drive_css(
        &mut self,
        computed: &ComputedStyles,
        tree: &RetainedTree,
        pseudo: &PseudoStates,
    ) -> bool {
        if self.css_motion.is_empty() {
            return false;
        }

        // Stage 1: collect. Pure reads of `self.css_motion`, `self.tree`,
        // `self.pseudo`, and the change-detection baselines.
        let mut target_updates: Vec<((NodeId, PropertyId), MotionValue)> = Vec::new();
        let mut transitions_to_start: Vec<TransitionStart> = Vec::new();
        let mut animations_to_start: Vec<AnimationStart> = Vec::new();

        for rule in self.css_motion.transitions() {
            let Some(kind) = animated_kind_for(rule.property()) else {
                continue;
            };
            for (id, node) in tree.iter() {
                if !node.kind().is_element() {
                    continue;
                }
                let element = TreeElement::with_pseudo(tree, id, pseudo);
                if SelectorMatcher::matches_list(rule.selectors(), &element).is_none() {
                    continue;
                }
                let Some(new_value) = computed
                    .get(&id)
                    .and_then(|style| motion_value_of(style, rule.property(), kind))
                else {
                    continue;
                };
                let key = (id, rule.property());
                match self.transition_targets.get(&key) {
                    // A real change of a watched property: start a transition from
                    // the previous target to the new one, unless the duration is
                    // non-positive (then it would just snap).
                    Some(&previous) if previous != new_value => {
                        target_updates.push((key, new_value));
                        if rule.duration_secs() > 0.0 {
                            // css-transitions-1 §3.1: if this change reverses an
                            // in-flight transition, shorten the new duration (and a
                            // negative delay) by the reversing shortening factor.
                            let (duration, delay, run) =
                                self.resolve_reversing(key, previous, new_value, rule);
                            transitions_to_start.push(TransitionStart {
                                node: id,
                                property: rule.property(),
                                from: previous,
                                to: new_value,
                                duration,
                                delay,
                                easing: rule.easing(),
                                kind,
                                run,
                            });
                        }
                    }
                    // Unchanged: nothing to do.
                    Some(_) => {}
                    // First observation: record the baseline, do not animate.
                    None => target_updates.push((key, new_value)),
                }
            }
        }

        for rule in self.css_motion.keyframes() {
            for (id, node) in tree.iter() {
                if !node.kind().is_element() {
                    continue;
                }
                let element = TreeElement::with_pseudo(tree, id, pseudo);
                if SelectorMatcher::matches_list(rule.selectors(), &element).is_none() {
                    continue;
                }
                for (property, track) in rule.tracks() {
                    let Some(kind) = animated_kind_for(*property) else {
                        continue;
                    };
                    if self.css_animations_started.contains(&(id, *property)) {
                        continue;
                    }
                    animations_to_start.push(AnimationStart {
                        node: id,
                        property: *property,
                        track: track.clone(),
                        duration: rule.duration_secs(),
                        kind,
                        playback: MotionPlayback {
                            delay_secs: rule.delay_secs(),
                            repeat: rule.repeat(),
                            direction: rule.direction(),
                            alternates: rule.alternates(),
                            fill: rule.fill(),
                        },
                    });
                }
            }
        }

        // Stage 2: apply. Mutating `self` (baselines + `animate`).
        // Read before the vectors are consumed: whether anything started is what
        // tells the caller this frame owes an animation frame.
        let started = !transitions_to_start.is_empty() || !animations_to_start.is_empty();
        for (key, value) in target_updates {
            self.transition_targets.insert(key, value);
        }
        for start in transitions_to_start {
            let track = KeyframeTrack::from_keyframes(vec![
                Keyframe::new(0.0, start.from).with_easing(start.easing),
                Keyframe::new(1.0, start.to),
            ]);
            self.transition_runs
                .insert((start.node, start.property), start.run);
            self.animate_with(
                start.node,
                start.property,
                track,
                start.duration,
                start.kind,
                TransitionInterruption::Blend,
                MotionPlayback::delayed(start.delay),
            );
        }
        for start in animations_to_start {
            self.css_animations_started
                .insert((start.node, start.property));
            self.animate_with(
                start.node,
                start.property,
                start.track,
                start.duration,
                start.kind,
                TransitionInterruption::Restart,
                start.playback,
            );
        }
        started
    }

    #[doc = include_str!("motion_driver.doc/resolve_reversing.md")]
    fn resolve_reversing(
        &self,
        key: (NodeId, PropertyId),
        previous: MotionValue,
        after: MotionValue,
        rule: &CssTransitionRule,
    ) -> (f32, f32, TransitionRun) {
        let matching_duration = rule.duration_secs();
        let matching_delay = rule.delay_secs();
        let easing = rule.easing();

        if let Some(old) = self.transition_runs.get(&key)
            && old.reversing_adjusted_start == after
            && let Some(old_progress) = self.running_transition_progress(key)
        {
            // Reversal: shorten by the new reversing shortening factor.
            let old_output = old.easing.apply(old_progress);
            let factor = (old_output * old.reversing_factor + (1.0 - old.reversing_factor))
                .abs()
                .clamp(0.0, 1.0);
            let delay = if matching_delay < 0.0 {
                matching_delay * factor
            } else {
                matching_delay
            };
            let run = TransitionRun {
                // The interrupted transition's end value is the new logical start
                // state, so repeated reversals compose (spec §3.1).
                reversing_adjusted_start: old.end_value,
                reversing_factor: factor,
                easing,
                end_value: after,
            };
            return (matching_duration * factor, delay, run);
        }

        // Fresh or non-reversing: full duration, factor 1. The reversing-adjusted
        // start value is the transition's start value — the current value of a
        // running transition, else the before-change value.
        let start_value = self.running_transition_value(key).unwrap_or(previous);
        let run = TransitionRun {
            reversing_adjusted_start: start_value,
            reversing_factor: 1.0,
            easing,
            end_value: after,
        };
        (matching_duration, matching_delay, run)
    }

    #[doc = include_str!("motion_driver.doc/running_transition_progress.md")]
    fn running_transition_progress(&self, key: (NodeId, PropertyId)) -> Option<f32> {
        let binding = self
            .animations
            .iter()
            .find(|b| b.node() == key.0 && b.property() == key.1)?;
        Some(self.timeline_progress(binding.timeline()))
    }

    #[doc = include_str!("motion_driver.doc/running_transition_value.md")]
    fn running_transition_value(&self, key: (NodeId, PropertyId)) -> Option<MotionValue> {
        let binding = self
            .animations
            .iter()
            .find(|b| b.node() == key.0 && b.property() == key.1)?;
        let progress = self.timeline_progress(binding.timeline());
        MotionCoordinator::sample_keyframes(binding.track(), progress)
    }
}

#[doc = include_str!("motion_driver.doc/TransitionStart.md")]
struct TransitionStart {
    node: NodeId,
    property: PropertyId,
    from: MotionValue,
    to: MotionValue,
    #[doc = include_str!("motion_driver.doc/duration.md")]
    duration: f32,
    #[doc = include_str!("motion_driver.doc/delay.md")]
    delay: f32,
    easing: obscur_motion::track::Easing,
    kind: AnimatedKind,
    #[doc = include_str!("motion_driver.doc/run.md")]
    run: TransitionRun,
}

#[doc = include_str!("motion_driver.doc/TransitionRun.md")]
#[derive(Debug, Clone, Copy)]
struct TransitionRun {
    #[doc = include_str!("motion_driver.doc/reversing_adjusted_start.md")]
    reversing_adjusted_start: MotionValue,
    #[doc = include_str!("motion_driver.doc/reversing_factor.md")]
    reversing_factor: f32,
    #[doc = include_str!("motion_driver.doc/easing.md")]
    easing: obscur_motion::track::Easing,
    #[doc = include_str!("motion_driver.doc/end_value.md")]
    end_value: MotionValue,
}

#[doc = include_str!("motion_driver.doc/AnimationStart.md")]
struct AnimationStart {
    node: NodeId,
    property: PropertyId,
    track: KeyframeTrack,
    duration: f32,
    kind: AnimatedKind,
    #[doc = include_str!("motion_driver.doc/playback.md")]
    playback: MotionPlayback,
}

#[doc = include_str!("motion_driver.doc/animated_kind_for.md")]
fn animated_kind_for(property: PropertyId) -> Option<AnimatedKind> {
    match property {
        PropertyId::Opacity | PropertyId::FlexGrow | PropertyId::FlexShrink => {
            Some(AnimatedKind::Number)
        }
        PropertyId::Width
        | PropertyId::Height
        | PropertyId::MinWidth
        | PropertyId::MinHeight
        | PropertyId::MaxWidth
        | PropertyId::MaxHeight
        | PropertyId::MarginTop
        | PropertyId::MarginRight
        | PropertyId::MarginBottom
        | PropertyId::MarginLeft
        | PropertyId::PaddingTop
        | PropertyId::PaddingRight
        | PropertyId::PaddingBottom
        | PropertyId::PaddingLeft
        | PropertyId::Gap
        | PropertyId::FlexBasis
        | PropertyId::FontSize => Some(AnimatedKind::LengthPx),
        // `transform` animates as post-layout motion (spec §4.17). The motion
        // engine interpolates it by matrix decomposition (CSS Transforms §14) —
        // `MotionValue::Transform` — not as a scalar.
        PropertyId::Transform => Some(AnimatedKind::Transform),
        // Colors animate with premultiplied-alpha interpolation (css-color-4
        // §13.3) via `MotionValue::Color`, not as a scalar.
        PropertyId::Color
        | PropertyId::BackgroundColor
        | PropertyId::BorderTopColor
        | PropertyId::BorderRightColor
        | PropertyId::BorderBottomColor
        | PropertyId::BorderLeftColor
        | PropertyId::OutlineColor => Some(AnimatedKind::Color),
        _ => None,
    }
}

#[doc = include_str!("motion_driver.doc/is_presentation_property.md")]
fn is_presentation_property(property: PropertyId) -> bool {
    matches!(property, PropertyId::Opacity | PropertyId::Transform)
}

#[doc = include_str!("motion_driver.doc/lower_presentation.md")]
fn lower_presentation(property: PropertyId, value: MotionValue) -> Option<PropertyValue> {
    match property {
        PropertyId::Transform => value
            .as_transform()
            .map(|matrix| PropertyValue::Transform(Transform::new(matrix))),
        _ => value.as_scalar().map(PropertyValue::Number),
    }
}

#[doc = include_str!("motion_driver.doc/motion_value_of.md")]
fn motion_value_of(
    style: &ComputedStyle,
    property: PropertyId,
    kind: AnimatedKind,
) -> Option<MotionValue> {
    let value = style.get(property)?;
    match kind {
        AnimatedKind::Number => match value.value() {
            PropertyValue::Number(n) => Some(MotionValue::scalar(*n)),
            _ => None,
        },
        AnimatedKind::LengthPx => value
            .as_length()
            .and_then(Length::as_px)
            .map(|px| MotionValue::scalar(px.0)),
        AnimatedKind::Transform => value
            .as_transform()
            .map(|transform| MotionValue::transform(transform.matrix())),
        AnimatedKind::Color => value
            .as_color()
            .map(|c| MotionValue::color([c.r, c.g, c.b, c.a])),
    }
}

#[cfg(test)]
mod tests {
    use super::MotionDriver;
    use crate::orchestration::{AnimatedKind, ComputedStyles, MotionPlayback};
    use obscur_identity::NodeId;
    use obscur_motion::track::{Keyframe, KeyframeTrack};
    use obscur_motion::transition::TransitionInterruption;
    use obscur_motion::value::MotionValue;
    use obscur_style::property::PropertyId;

    fn track(from: f32, to: f32) -> KeyframeTrack {
        KeyframeTrack::from_keyframes(vec![
            Keyframe::new(0.0, MotionValue::Scalar(from)),
            Keyframe::new(1.0, MotionValue::Scalar(to)),
        ])
    }

    fn start(driver: &mut MotionDriver, node: NodeId) {
        driver.animate_with(
            node,
            PropertyId::Width,
            track(0.0, 100.0),
            1.0,
            AnimatedKind::LengthPx,
            TransitionInterruption::Restart,
            MotionPlayback::default(),
        );
    }

    #[test]
    fn a_fresh_driver_runs_nothing_and_asks_for_no_frame() {
        let mut driver = MotionDriver::new();
        assert_eq!(driver.active_animations(), 0);
        assert_eq!(driver.binding_count(), 0);
        assert!(!driver.advance(0.5), "nothing running asks for no frame");
    }

    #[test]
    fn a_timeline_stops_asking_for_frames_once_it_finishes() {
        let mut driver = MotionDriver::new();
        start(&mut driver, NodeId::from_raw(1));
        assert_eq!(driver.active_animations(), 1);
        assert!(driver.advance(0.5), "still running");
        // Past the end: the scheduler prunes it and stops asking.
        assert!(!driver.advance(1.0));
        assert_eq!(driver.active_animations(), 0);
        // The binding survives the prune, because the animation holds its final
        // value until something supersedes it.
        assert_eq!(driver.binding_count(), 1);
    }

    #[test]
    fn a_second_animation_of_one_target_supersedes_the_first() {
        let mut driver = MotionDriver::new();
        let node = NodeId::from_raw(1);
        start(&mut driver, node);
        start(&mut driver, node);
        // One target, one binding: the stale timeline and its settled progress go
        // with the binding it belonged to.
        assert_eq!(driver.binding_count(), 1);
        assert_eq!(driver.active_animations(), 1);
        // A different property is a different target and coexists.
        driver.animate_with(
            node,
            PropertyId::Height,
            track(0.0, 50.0),
            1.0,
            AnimatedKind::LengthPx,
            TransitionInterruption::Restart,
            MotionPlayback::default(),
        );
        assert_eq!(driver.binding_count(), 2);
    }

    #[test]
    fn a_settled_animation_still_overlays_its_endpoint() {
        let mut driver = MotionDriver::new();
        let node = NodeId::from_raw(1);
        start(&mut driver, node);
        driver.advance(2.0);
        assert_eq!(driver.active_animations(), 0, "finished and pruned");

        let mut computed = ComputedStyles::new();
        computed.insert(node, obscur_style::computed::ComputedStyle::default());
        let mut overlaid = ComputedStyles::new();
        driver.apply_overlay(&mut computed, &mut overlaid);
        // The endpoint keeps being applied, and the pre-motion cascade value was
        // stashed for the style phase to put back.
        assert!(computed.contains_key(&node));
        assert!(
            overlaid.contains_key(&node),
            "the settled cascade value is stashed exactly once"
        );
    }

    #[test]
    fn installing_rules_clears_all_three_baselines_together() {
        let mut driver = MotionDriver::new();
        // Nothing installed: the driver reports no CSS motion to drive at all.
        assert!(!driver.drive_css(
            &ComputedStyles::new(),
            &obscur_tree::tree::RetainedTree::new(),
            &crate::orchestration::PseudoStates::new(),
        ));
    }
}
