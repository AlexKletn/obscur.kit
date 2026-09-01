#![doc = include_str!("motion_driver.doc/_module.md")]

use std::collections::{HashMap, HashSet};

use super::PresentationUndo;
use obscur_identity::{FastMap, FastSet, NodeId};
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
    settled_progress: FastMap<TimelineId, f32>,
    #[doc = include_str!("motion_driver.doc/css_motion.md")]
    css_motion: CssMotionRules,
    #[doc = include_str!("motion_driver.doc/transition_targets.md")]
    transition_targets: FastMap<(NodeId, PropertyId), MotionValue>,
    #[doc = include_str!("motion_driver.doc/transition_runs.md")]
    transition_runs: FastMap<(NodeId, PropertyId), TransitionRun>,
    #[doc = include_str!("motion_driver.doc/css_animations_started.md")]
    css_animations_started: FastSet<(NodeId, PropertyId)>,
}

impl MotionDriver {
    #[doc = include_str!("motion_driver.doc/new.md")]
    pub(crate) fn new() -> Self {
        Self {
            motion: MotionScheduler::new(),
            animations: Vec::new(),
            presentation_overlays: Vec::new(),
            next_timeline: TimelineId::from_raw(0),
            settled_progress: HashMap::default(),
            css_motion: CssMotionRules::new(),
            transition_targets: HashMap::default(),
            transition_runs: HashMap::default(),
            css_animations_started: HashSet::default(),
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

    #[doc = include_str!("motion_driver.doc/bookkeeping_counts.md")]
    #[cfg(test)]
    pub(crate) fn bookkeeping_counts(&self) -> (usize, usize, usize, usize) {
        (
            self.settled_progress.len(),
            self.transition_targets.len(),
            self.transition_runs.len(),
            self.css_animations_started.len(),
        )
    }

    #[doc = include_str!("motion_driver.doc/retain_live.md")]
    fn retain_live(&mut self, tree: &RetainedTree) {
        // Borrowed out of `self` first so the retain closure below touches only
        // the scheduler field: `animations` and `motion` are disjoint, which is
        // what lets a binding pull its own timeline out as it goes.
        let motion = &mut self.motion;
        self.animations.retain(|binding| {
            // A `NodeId` is generational, so a remounted node never matches the
            // dead entry — nothing here can ever be reclaimed by coincidence, and
            // an unmatched binding is dead for good.
            if tree.contains(binding.node()) {
                return true;
            }
            // The timeline goes with the binding it belonged to. Left behind, it
            // would keep `needs_frame` true forever and the runtime would render
            // frames to advance an animation no node can show.
            motion.remove(binding.timeline());
            false
        });
        // Reachable only through a surviving binding's `timeline()`, so the
        // bindings that just survived are the whole of what can still be read.
        let live: FastSet<TimelineId> = self
            .animations
            .iter()
            .map(AnimationBinding::timeline)
            .collect();
        self.settled_progress.retain(|id, _| live.contains(id));
        self.transition_targets
            .retain(|(node, _), _| tree.contains(*node));
        self.transition_runs
            .retain(|(node, _), _| tree.contains(*node));
        self.css_animations_started
            .retain(|(node, _)| tree.contains(*node));
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

    #[doc = include_str!("motion_driver.doc/apply_presentation.md")]
    pub(crate) fn apply_presentation(&self, computed: &mut ComputedStyles) -> PresentationUndo {
        // The map is NOT cloned. It used to be, and one live transition anywhere
        // in the tree was enough to deep-copy every node's style — two hash maps
        // per node, each owning heap for font stacks, gradient stops, shadows and
        // filters — on every frame of the animation, to record a handful of
        // samples. What is written here is exactly what the overlays name, and
        // `undo` carries precisely enough to put it back.
        let mut undo = PresentationUndo::default();
        for (node, property, value) in &self.presentation_overlays {
            let Some(sampled) = ComputedValue::from_concrete(value.clone()) else {
                continue;
            };
            let fresh = !computed.contains_key(node);
            let style = computed.entry(*node).or_default();
            let previous = style.set(*property, sampled);
            undo.entries.push((*node, *property, previous, fresh));
        }
        undo
    }

    #[doc = include_str!("motion_driver.doc/restore_presentation.md")]
    pub(crate) fn restore_presentation(computed: &mut ComputedStyles, undo: PresentationUndo) {
        // Reverse order, because two overlays on the same property would
        // otherwise restore the older value over the newer one.
        for (node, property, previous, fresh) in undo.entries.into_iter().rev() {
            if fresh {
                computed.remove(&node);
                continue;
            }
            let Some(style) = computed.get_mut(&node) else {
                continue;
            };
            match previous {
                Some(value) => style.set(property, value),
                None => style.remove(property),
            };
        }
    }

    #[doc = include_str!("motion_driver.doc/drive_css.md")]
    pub(crate) fn drive_css(
        &mut self,
        computed: &ComputedStyles,
        tree: &RetainedTree,
        pseudo: &PseudoStates,
    ) -> bool {
        // Swept against the tree this frame's cascade was resolved from, in the
        // pass that already holds it — this costs one hash lookup per live
        // binding, not a second walk. It runs BEFORE the `is_empty` return
        // because an imperatively started animation (`UiRuntime::animate`) has no
        // CSS rule behind it and would otherwise never be swept at all. Without
        // it every list row, dialog and toast that ever animated leaves its
        // binding behind, and `apply_overlay` walks all of them every frame: a
        // leak that shows up as frame time before it shows up as memory.
        self.retain_live(tree);
        if self.css_motion.is_empty() {
            return false;
        }

        // Stage 1: collect. Pure reads of `self.css_motion`, `self.tree`,
        // `self.pseudo`, and the change-detection baselines.
        let mut target_updates: Vec<((NodeId, PropertyId), MotionValue)> = Vec::new();
        let mut transitions_to_start: Vec<TransitionStart> = Vec::new();
        let mut animations_to_start: Vec<AnimationStart> = Vec::new();

        // Node outer, rule inner. `TreeElement::with_pseudo` walks to build the
        // element a selector is matched against, and with the loops the other way
        // round it was rebuilt once per (rule, node) pair — and a single
        // `transition: a .1s, b .1s, c .1s` is three rules, because a
        // `CssTransitionRule` carries one property. Now it is built once per node,
        // whatever the sheet declares.
        for (id, node) in tree.iter() {
            if !node.kind().is_element() {
                continue;
            }
            let element = TreeElement::with_pseudo(tree, id, pseudo);
            for rule in self.css_motion.transitions() {
                let Some(kind) = animated_kind_for(rule.property()) else {
                    continue;
                };
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

            for rule in self.css_motion.keyframes() {
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
    use crate::orchestration::{
        AnimatedKind, ComputedStyles, CssKeyframeRule, CssMotionRules, CssTransitionRule,
        MotionPlayback, PseudoStates,
    };
    use obscur_identity::{ComponentId, NodeId};
    use obscur_motion::track::{Easing, Keyframe, KeyframeTrack};
    use obscur_motion::transition::TransitionInterruption;
    use obscur_motion::value::MotionValue;
    use obscur_style::computed::{ComputedStyle, ComputedValue};
    use obscur_style::property::{PropertyId, PropertyValue};
    use obscur_style::selector::{ComplexSelector, CompoundSelector, SelectorList, SimpleSelector};
    use obscur_tree::node::NodeKind;
    use obscur_tree::tree::RetainedTree;

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

        let mut computed = ComputedStyles::default();
        computed.insert(node, obscur_style::computed::ComputedStyle::default());
        let mut overlaid = ComputedStyles::default();
        driver.apply_overlay(&mut computed, &mut overlaid);
        // The endpoint keeps being applied, and the pre-motion cascade value was
        // stashed for the style phase to put back.
        assert!(computed.contains_key(&node));
        assert!(
            overlaid.contains_key(&node),
            "the settled cascade value is stashed exactly once"
        );
    }

    /// `*` — every element matches, so the rules below apply to whatever the
    /// tree happens to hold and the test is about lifetime, not selection.
    fn universal() -> SelectorList {
        SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Universal]),
            Vec::new(),
        )])
    }

    fn number(style: &mut ComputedStyle, property: PropertyId, n: f32) {
        style.set(
            property,
            ComputedValue::from_concrete(PropertyValue::Number(n)).expect("a concrete number"),
        );
    }

    #[test]
    fn an_unmounted_node_takes_its_animation_binding_with_it() {
        // A list row, a dialog, a toast: mounted, animated, unmounted. Without the
        // sweep its binding lives for the life of the process — and `apply_overlay`
        // walks the whole vector every style phase, so this is frame time before it
        // is memory. The assertion is the vector's LENGTH for exactly that reason.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let mut driver = MotionDriver::new();
        let empty = ComputedStyles::default();
        let pseudo = PseudoStates::new();

        for _ in 0..50 {
            let row = tree.alloc(NodeKind::Element {
                component: ComponentId::from_raw(1),
            });
            start(&mut driver, row);
            assert_eq!(driver.binding_count(), 1, "one row, one binding");
            tree.remove(row);
            driver.drive_css(&empty, &tree, &pseudo);
            assert_eq!(
                driver.binding_count(),
                0,
                "the unmounted row left its binding behind"
            );
        }
        // The scheduler got its timelines back too: left in it, they would keep
        // asking for frames forever.
        assert_eq!(driver.active_animations(), 0);
        assert_eq!(driver.bookkeeping_counts().0, 0, "settled_progress");
    }

    #[test]
    fn an_unmounted_node_takes_its_transition_bookkeeping_with_it() {
        // The three `(NodeId, PropertyId)` maps CSS motion keeps. They are cleared
        // only when the stylesheet set is replaced, which for a running program is
        // never — so each is one entry per (node, property) ever animated.
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let node = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(1),
        });

        let mut driver = MotionDriver::new();
        driver.set_css_rules(CssMotionRules::from_parts(
            vec![CssTransitionRule::new(
                universal(),
                PropertyId::FlexGrow,
                1.0,
                Easing::Linear,
            )],
            vec![CssKeyframeRule::new(
                universal(),
                vec![(PropertyId::FlexShrink, track(0.0, 1.0))],
                1.0,
            )],
        ));
        let pseudo = PseudoStates::new();

        let mut style = ComputedStyle::default();
        number(&mut style, PropertyId::FlexGrow, 0.0);
        let mut computed = ComputedStyles::default();
        computed.insert(node, style.clone());
        // First observation records the baseline and starts the keyframe animation.
        driver.drive_css(&computed, &tree, &pseudo);
        // A real change of the watched property starts the transition and records
        // its reversing run.
        number(&mut style, PropertyId::FlexGrow, 1.0);
        computed.insert(node, style);
        driver.drive_css(&computed, &tree, &pseudo);

        let (_, targets, runs, started) = driver.bookkeeping_counts();
        assert_eq!((targets, runs, started), (1, 1, 1), "all three recorded");
        assert_eq!(
            driver.binding_count(),
            2,
            "the transition and the animation"
        );

        tree.remove(node);
        driver.drive_css(&computed, &tree, &pseudo);
        assert_eq!(
            driver.bookkeeping_counts(),
            (0, 0, 0, 0),
            "the unmounted node left nothing behind"
        );
        assert_eq!(driver.binding_count(), 0);
    }

    #[test]
    fn installing_rules_clears_all_three_baselines_together() {
        let mut driver = MotionDriver::new();
        // Nothing installed: the driver reports no CSS motion to drive at all.
        assert!(!driver.drive_css(
            &ComputedStyles::default(),
            &obscur_tree::tree::RetainedTree::new(),
            &crate::orchestration::PseudoStates::new(),
        ));
    }
}
