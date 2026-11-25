//! The frame's style phase, whole: [`StylePhase`].

use obscur_diagnostics::DiagnosticSink;
use obscur_identity::{FastMap, NodeId};
use obscur_style::matching::{
    ContainerQueryEnvironment, MediaEnvironment, MediaInput, MediaPreferences, Stylesheet,
};
use obscur_style::property::EnvironmentValues;
use obscur_text::shaping::ParagraphShaper;
use obscur_tree::tree::RetainedTree;

use super::{
    AuthorSheets, CascadeSheets, FramePhase, FramePhaseClock, FrameTelemetry, MotionDriver,
    RestylePhase, StyleDelta, StyleKey,
};
use crate::frame::FrameContext;
use crate::orchestration::{
    ComputedStyles, GeneratedContent, InlineStyles, PseudoStates, StyleCoordinator, StyleInputs,
    StyleMatchingCoordinator, resolve_generated_content,
};

#[doc = include_str!("../../docs/api/runtime/style_phase.doc/StylePhase.md")]
pub(crate) struct StylePhase<'a> {
    /// The retained tree the sheets are matched against.
    tree: &'a RetainedTree,
    /// The live per-node interaction state, so `:hover`/`:focus` participate.
    pseudo: &'a PseudoStates,
    /// The host-reported environment values `env()` resolves against (§6.15).
    environment: EnvironmentValues,
    media_preferences: MediaPreferences,
    media_input: Option<MediaInput>,
    device_scale: f32,
    query_containers: &'a ContainerQueryEnvironment,
    /// The shaper that folds `ch` during resolution — the same one layout and
    /// compose measure and paint with.
    paragraph: &'a ParagraphShaper,
    /// Matches authored stylesheets (pseudo-state aware) into cascade inputs.
    matcher: &'a mut StyleMatchingCoordinator,
    /// Resolves computed styles over the retained tree.
    style: &'a mut StyleCoordinator,
    /// The cascade inputs this frame produces and the runtime retains.
    inputs: &'a mut StyleInputs,
    /// The settled cascade value of every node the motion overlay has written
    /// over, stashed before the overwrite so change detection and an incremental
    /// restyle both see the resolver's own output.
    overlaid: &'a mut ComputedStyles,
    /// The `style` attributes, compiled and cached.
    inline: &'a mut InlineStyles,
    /// The computed styles retained from the last styled frame: read as the
    /// incremental path's baseline, and **moved out** when it takes that path.
    retained: &'a mut ComputedStyles,
    /// What the retained inputs/styles were produced from, rewritten here so the
    /// next frame can tell whether interaction state is all that moved.
    last_style: &'a mut Option<StyleKey>,
    #[doc = include_str!("../../docs/api/runtime/style_phase.doc/generated_content.md")]
    generated_content: &'a mut FastMap<NodeId, GeneratedContent>,
    /// Every source a frame's stylesheets come from; the only thing that may put
    /// the UA sheet and the author sheets into one ordered list.
    sheets: &'a CascadeSheets,
    /// The timelines whose CSS rules are driven here and whose samples are laid
    /// over the resolved styles.
    motion: &'a mut MotionDriver,
    /// Where the style phase's span and resolved-style count land (spec §7.9).
    telemetry: &'a mut FrameTelemetry,
    /// Where a cascade diagnostic is reported (spec §1.6).
    diagnostics: &'a dyn DiagnosticSink,
}

impl<'a> StylePhase<'a> {
    /// Bundles the state one style phase reads and writes.
    // A borrow bundle's constructor is exactly as wide as the state the phase
    // touches, and this phase is the widest of the four: the cascade tier, the two
    // retained maps, the two records that justify the next frame's reuse, and
    // motion. Folding them into a struct would only move the same fifteen names.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        tree: &'a RetainedTree,
        pseudo: &'a PseudoStates,
        environment: EnvironmentValues,
        media_preferences: MediaPreferences,
        media_input: Option<MediaInput>,
        device_scale: f32,
        query_containers: &'a ContainerQueryEnvironment,
        paragraph: &'a ParagraphShaper,
        matcher: &'a mut StyleMatchingCoordinator,
        style: &'a mut StyleCoordinator,
        inputs: &'a mut StyleInputs,
        overlaid: &'a mut ComputedStyles,
        inline: &'a mut InlineStyles,
        retained: &'a mut ComputedStyles,
        last_style: &'a mut Option<StyleKey>,
        generated_content: &'a mut FastMap<NodeId, GeneratedContent>,
        sheets: &'a CascadeSheets,
        motion: &'a mut MotionDriver,
        telemetry: &'a mut FrameTelemetry,
        diagnostics: &'a dyn DiagnosticSink,
    ) -> Self {
        Self {
            tree,
            pseudo,
            environment,
            media_preferences,
            media_input,
            device_scale,
            query_containers,
            paragraph,
            matcher,
            style,
            inputs,
            overlaid,
            inline,
            retained,
            last_style,
            generated_content,
            sheets,
            motion,
            telemetry,
            diagnostics,
        }
    }

    #[doc = include_str!("../../docs/api/runtime/style_phase.doc/run.md")]
    pub(crate) fn run(
        &mut self,
        ctx: &FrameContext,
        restyle: bool,
        author_sheets: AuthorSheets,
        reconcile_roots: Option<&[NodeId]>,
        reconciled: bool,
        phase_clock: &FramePhaseClock,
    ) -> (ComputedStyles, bool, StyleDelta) {
        if !restyle {
            // Reuse the retained computed styles; re-apply the motion overlay so
            // a motion-only frame still advances its animated values.
            let mut computed = self.retained.clone();
            self.motion.apply_overlay(&mut computed, self.overlaid);
            return (computed, false, StyleDelta::Full);
        }
        // Match authored sheets against the tree, honouring live interaction
        // state so `:hover`/`:focus` participate and gating `@media`/
        // `@container` rules by the latest arranged ancestor sizes (§4.13).
        let environment = MediaEnvironment::viewport(ctx.viewport().width, ctx.viewport().height)
            .with_preferences(self.media_preferences)
            .with_input(self.media_input)
            .with_resolution_dppx(self.device_scale)
            .with_query_containers(self.query_containers);
        // The list this frame cascades: the UA default sheet (if installed)
        // first — below every author sheet, since it carries the lowest cascade
        // origin (`SystemDefault`) and any author declaration overrides it
        // (§4.14) — then the author sheets retention resolved above. Only
        // [`CascadeSheets`] can produce it, which is what keeps the retained
        // author set free of a UA sheet that a later `set_default_stylesheet`
        // would then be unable to replace.
        let sheets: Vec<Stylesheet> = self.sheets.cascade(author_sheets);
        // Phase 5 (§3.4), per node: does this frame have to restyle everything,
        // or does it know exactly which nodes can differ? `None` means it does
        // not know and must run the whole pass.
        phase_clock.mark_reconcile();
        // The cascade tier is borrowed only for the match + resolve itself, so the
        // motion overlay below can take `overlaid` back.
        let (mut computed, resolved_count, delta, dirty) = {
            let mut phase = RestylePhase::new(
                self.tree,
                self.pseudo,
                self.environment,
                self.paragraph,
                self.matcher,
                self.style,
                self.inputs,
                self.overlaid,
                self.inline,
            );
            let scope = match reconcile_roots {
                Some(roots) => phase.scope(
                    self.last_style.as_ref(),
                    self.retained,
                    &sheets,
                    ctx.viewport(),
                    roots,
                ),
                // The reconcile could not be bounded (a removal, or a component
                // sub-reconcile): restyle everything.
                None => None,
            };
            // The dirty set outlives the match: the generated-content pass below
            // is scoped by the same set the cascade was.
            if let Some(dirty) = scope {
                // The retained map is moved out and handed back as this frame's
                // styles, so a node outside the dirty set costs nothing at all.
                let (computed, resolved, delta) = phase.invalidated(
                    std::mem::take(self.retained),
                    &sheets,
                    &environment,
                    ctx.viewport(),
                    &dirty,
                    self.diagnostics,
                    reconciled,
                );
                (computed, resolved, delta, Some(dirty))
            } else {
                let (computed, resolved, delta) =
                    phase.everything(&sheets, &environment, ctx.viewport(), self.diagnostics);
                (computed, resolved, delta, None)
            }
        };
        // §4.16 generated / synthetic decoration (`::before`/`::after` content
        // and the input placeholder) is resolved from the SAME sheets (UA +
        // author) and environment the cascade used, so compose can paint it and
        // `inspect_snapshot` can project the synthetic placeholder part. It is
        // pure decoration. Arrange and compose project it privately, out of the
        // application tree, accessibility and hit testing. It is scoped
        // the same way the cascade above was: an incremental frame resolves the
        // dirty nodes and keeps the previous frame's answer for the rest, and
        // the pass skips the selector engine entirely for a sheet set that
        // authors no decoration — which is most of them.
        resolve_generated_content(
            self.generated_content,
            self.tree,
            &sheets,
            &environment,
            dirty.as_ref(),
            self.pseudo,
        );
        // Remember what these inputs/styles were produced from, so the next
        // frame can tell whether interaction state is all that moved.
        *self.last_style = Some(StyleKey {
            viewport: ctx.viewport(),
            environment: ctx.environment(),
            sheets,
            pseudo: self.pseudo.clone(),
        });
        // 3a. CSS motion: match selector-bound `transition`/`@keyframes`
        //     rules against the tree over the *resolved* (pre-overlay) values,
        //     starting a timeline for any transitioned property that changed
        //     or any newly-matched animation (spec §4.17 S4).
        let style_inputs_changed = reconciled
            || resolved_count != 0
            || dirty.as_ref().is_none_or(|dirty| !dirty.is_empty());
        self.motion.drive_css(
            &computed,
            self.tree,
            self.pseudo,
            &environment,
            style_inputs_changed,
        );
        // 3b. Motion: overlay every active animation's sampled value onto the
        //     computed style *before* layout/paint (spec §4.17 style →
        //     motion → paint). This now includes the CSS-started timelines.
        self.motion.apply_overlay(&mut computed, self.overlaid);
        phase_clock.record_style_scope(resolved_count as u64, self.tree.len() as u64);
        let restyled = !computed.is_empty();
        // Telemetry: mark the style phase and count the styles this frame
        // actually resolved — NOT the size of the resulting map (§7.9). On the
        // incremental path those differ, and the whole point is which one.
        let count = resolved_count as u64;
        self.telemetry
            .record_phase(FramePhase::Style, count, ctx.id(), self.diagnostics);
        (computed, restyled, delta)
    }
}
