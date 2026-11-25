//! Emitting a text node's glyph runs — and, for the focused editable, its
//! selection highlight, IME preedit underline and caret — into the node's own
//! paint unit (spec §5.10, §5.11, §5.12).

use obscur_geometry::{Point, Rect, Size};
use obscur_identity::NodeId;
use obscur_scene::color::Color;
use obscur_scene::command::DisplayListBuilder;
use obscur_scene::paint::Brush;
use obscur_scene::text::{GlyphRun, PositionedGlyph};
use obscur_text::selection::{SelectionGeometry, TextSelection};
use obscur_text::shaping::ShapedRun;
use obscur_tree::tree::RetainedTree;

use super::caret_geometry::{caret_line_geometry, caret_line_geometry_affinity};
use super::editing_role::{EditingRole, editing_role};
use super::paint_color::{scene_color_from, text_color_of};
use super::paragraph_shaping::{ParagraphShaping, font_size_of};
use super::{
    ComputedStyles, EditingView, GeneratedRun, LayoutBoxes, display_text, text_transform_for,
    white_space_for,
};

/// The width, in logical pixels, of the caret rectangle painted at the collapsed
/// selection of the focused editable (a thin vertical bar over the text baseline).
pub(crate) const CARET_WIDTH: f32 = 1.0;

/// The thickness, in logical pixels, of the underline painted beneath an active
/// IME preedit (spec §5.12) — the visual mark distinguishing the uncommitted
/// composing text from committed glyphs.
pub(super) const PREEDIT_UNDERLINE_THICKNESS: f32 = 1.0;

/// The default selection-highlight color: a semi-transparent blue painted behind
/// the selected span so the glyphs show through it (spec §5.11). A host has no
/// hook to override this yet; it is a documented fixed default.
pub(super) const SELECTION_COLOR: Color = Color::rgba(0.20, 0.45, 0.90, 0.35);

#[doc = include_str!("../../docs/api/orchestration/text_painter.doc/positioned_run.md")]
fn positioned_run(
    segment: &ShapedRun,
    baseline: f32,
    font_size: f32,
    text: &str,
    content: &str,
) -> GlyphRun {
    let mut pen_x = 0.0_f32;
    let run_glyphs = segment.run().glyphs();
    let mut glyphs: Vec<PositionedGlyph> = Vec::with_capacity(run_glyphs.len());
    // Asked once for the paragraph rather than once per glyph. A control
    // character shapes to a `.notdef` tofu and has to be suppressed, but running
    // text has none at all, and the check below sliced the string and decoded a
    // `char` for every glyph on screen to establish that.
    let any_control = content.chars().any(char::is_control);
    for glyph in run_glyphs {
        // Suppress the visible glyph but keep advancing the pen, so positions
        // and caret offsets hold.
        let suppressed = any_control
            && content
                .get(glyph.cluster()..)
                .and_then(|rest| rest.chars().next())
                .is_some_and(char::is_control);
        if !suppressed {
            glyphs.push(PositionedGlyph::new(
                glyph.glyph_id(),
                glyph.cluster(),
                pen_x + glyph.x_offset(),
                baseline + glyph.y_offset(),
                glyph.x_advance(),
            ));
        }
        pen_x += glyph.x_advance();
    }
    let has_suppressed_glyphs = glyphs.len() != run_glyphs.len();
    let run = GlyphRun::new(glyphs, font_size, text)
        .with_font(segment.font().get())
        .with_synthesis(segment.synth_oblique(), segment.synth_bold());
    if has_suppressed_glyphs {
        run.with_advance(pen_x)
    } else {
        run
    }
}

#[doc = include_str!("../../docs/api/orchestration/text_painter.doc/paint_text_node.md")]
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_text_node(
    shaping: &ParagraphShaping,
    node_paint: &mut DisplayListBuilder,
    tree: &RetainedTree,
    computed: &ComputedStyles,
    boxes: &LayoutBoxes,
    node: NodeId,
    retained_content: &str,
    editing: Option<EditingView<'_>>,
) {
    let Some(layout_box) = boxes.get(&node) else {
        return;
    };
    // This node's computed style, looked up ONCE for the whole paint. Every
    // `*_for` reader below took the map and the id and probed it again — a dozen
    // probes of one entry — and they all already accept the resolved style.
    let style = computed.get(&node);
    // Is this text node the focused editable's text? Then paint the
    // live SESSION text (not the possibly-lagging retained content)
    // and, below, the caret + selection highlight. `editing` is
    // `None` on every non-editing frame, so this whole override
    // path is inert and the retained content is drawn unchanged.
    // The role also decides the `.placeholder` overlay case: the
    // session overrides the placeholder slot only while it has typed
    // text (`:placeholder-shown` is false once the effective text is
    // non-empty), and then paints in the FIELD's ink, not the muted
    // hint color.
    let role = editing.map_or(EditingRole::None, |view| {
        editing_role(tree, node, view.node)
    });
    let active = editing.filter(|view| role.overrides(view));
    // The displayed text, computed through the SAME `display_text`
    // seam the layout (measure/arrange) pass uses, so this frame's
    // box and the glyphs painted here shape the identical string
    // (CSS Text 3 §4.1.1). While actively editing this node's text,
    // the live session buffer (with any IME preedit spliced in for
    // DISPLAY, spec §5.12) is shaped RAW — `text-transform` (§2.1)
    // and `white-space` collapsing (§4.1.1) are skipped so the
    // caret/selection can index the shaped string by live buffer
    // byte offsets; otherwise the retained content flows through
    // casing then whitespace processing. The field is wrapped by its
    // OWN `white-space` in both states, so a single-line `input`
    // (`nowrap`, per the UA sheet) stays one line while typing — it
    // does NOT reflow like a `<textarea>` — while a `pre-wrap` field
    // keeps its literal newlines and wraps at the box width.
    let white_space = white_space_for(style);
    let displayed = display_text(
        retained_content,
        active,
        text_transform_for(style),
        white_space,
    );
    let content: &str = displayed.as_ref();
    // The rendered caret/selection: while composing, the caret sits
    // at the preedit's caret within the SPLICED display text; with
    // no preedit it is the session's own selection (buffer offsets
    // equal display offsets). `None` on a non-editing text node.
    let display_selection = active.map(|view| match view.preedit {
        Some(preedit) => {
            let at = preedit.at.min(view.text.len());
            TextSelection::caret(at + preedit.caret)
        }
        None => view.selection,
    });
    // The ink for the glyphs, caret and preedit underline. Normally
    // the node's own computed `color`; but when the live session is
    // painted OVER the `.placeholder` overlay (typed text, not the
    // hint) it uses the editing FIELD's normal ink instead of the
    // overlay's muted placeholder color — so typed text is not muted.
    let text_color = match (active, role) {
        (Some(view), EditingRole::Placeholder) => text_color_of(computed.get(&view.node)),
        _ => text_color_of(style),
    };
    let brush = Brush::solid(text_color);
    // Same per-node font size the layout phase measured with, so
    // the drawn advances and baseline match the sized box.
    let font_size = font_size_of(computed, node);
    let origin = layout_box.origin();
    let scroll_left = active.map_or(0.0, |view| view.scroll_left);
    let box_width = layout_box.size().width;
    let layout = if active.is_some() {
        shaping.shape_editable_lines(tree, computed, boxes, node, content, box_width)
    } else {
        shaping.shape_text_lines(tree, computed, boxes, node, content, box_width)
    };
    // Selection highlight, painted BEHIND the glyphs (emitted before
    // the runs so the text shows over it): the documented
    // semi-transparent selection color. A span over the wrapped
    // `layout` paints ONE partial rect per covered line (spec §5.11):
    // the leading line from x(start) to its trailing edge, whole
    // interior lines full-width, the trailing line up to x(end). The
    // per-line geometry is computed by the domain
    // `SelectionGeometry::from_paragraph` (the source of truth) in
    // paragraph-local space using the SAME per-line spaced/tabbed
    // advance + alignment `x_offset` model the caret uses; the
    // coordinator only translates each rect by the box `origin` and
    // the field's `−scroll_left`. A span confined to ONE line yields
    // exactly the single rect the old first-line-only path produced
    // (byte-identical). A collapsed selection paints nothing.
    if let Some(selection) = display_selection
        && !selection.is_empty()
    {
        let geometry =
            SelectionGeometry::from_paragraph(&layout, selection.start(), selection.end());
        for line_rect in geometry.rects() {
            node_paint.fill_rect(
                Rect::new(
                    Point::new(
                        origin.x + line_rect.origin.x - scroll_left,
                        origin.y + line_rect.origin.y,
                    ),
                    line_rect.size,
                ),
                Brush::solid(SELECTION_COLOR),
            );
        }
    }
    for line in layout.lines() {
        let baseline = line.baseline();
        let line_top = origin.y + line.top();
        let line_left = origin.x + line.x_offset() - scroll_left;
        if line.segments().is_empty() {
            // An empty line (e.g. empty text) still emits a single
            // empty run at the line origin so the painted line count
            // matches the box height (one line per wrapped line).
            let range = line.range();
            let line_text = content.get(range.start..range.end).unwrap_or_default();
            let run = GlyphRun::new(Vec::new(), font_size, line_text);
            node_paint.draw_text(Point::new(line_left, line_top), run, brush.clone());
            continue;
        }
        // Walk the line's contiguous same-font segments left to
        // right, emitting one DrawText per segment at the running
        // pen origin so a multi-font line paints as several
        // font-tagged runs (spec §5.10).
        let mut seg_x = 0.0_f32;
        for segment in line.segments() {
            let range = segment.run().source();
            let seg_text = content.get(range.start..range.end).unwrap_or_default();
            let run = positioned_run(segment, baseline, font_size, seg_text, content);
            let seg_origin = Point::new(line_left + seg_x, line_top);
            node_paint.draw_text(seg_origin, run, brush.clone());
            seg_x += segment.run().advance();
        }
    }
    // IME preedit underline (spec §5.12): a thin bar beneath the
    // composing text, distinguishing the uncommitted preedit from
    // committed glyphs. The preedit's x-range is bounded with the
    // SAME line-aware, spaced advance-walk the caret/selection use
    // (`caret_line_geometry`), over the spliced display `content`:
    // from x(at) to x(at + preedit len), including the preedit line's
    // alignment `x_offset` so a right/centre-aligned field underlines
    // the composing glyphs. Only for the editing field with a live,
    // non-empty preedit.
    if let Some(preedit) = active.and_then(|view| view.preedit)
        && !preedit.text.is_empty()
    {
        let at = preedit.at.min(content.len());
        let line_height = shaping.paragraph().line_metrics(font_size).line_height();
        let (top0, _h0, xoff0, x0) = caret_line_geometry(shaping, &layout, font_size, at);
        let (_, _, _, x1) =
            caret_line_geometry(shaping, &layout, font_size, at + preedit.text.len());
        node_paint.fill_rect(
            Rect::new(
                Point::new(
                    origin.x + xoff0 + x0 - scroll_left,
                    origin.y + top0 + line_height - PREEDIT_UNDERLINE_THICKNESS,
                ),
                Size::new((x1 - x0).max(0.0), PREEDIT_UNDERLINE_THICKNESS),
            ),
            Brush::solid(text_color),
        );
    }
    // Caret, painted ON TOP of the glyphs (emitted after the runs):
    // a thin vertical bar at the caret's pen x, spanning the first
    // line's height, in the text color. Only for the editing field
    // and only when the (display) selection is collapsed (a caret) —
    // a non-empty selection shows the highlight above instead. While
    // composing, the display selection is the preedit's caret.
    if let Some(selection) = display_selection
        && selection.is_collapsed()
    {
        // Line-aware caret geometry (spec §5.10/§5.11): the focus
        // byte resolves to its wrapped line's top + height and its
        // pen x within that line, so a caret on line 2 paints at
        // y ≈ line-height and an x measured within line 2 — not the
        // summed width of every line. At a soft-wrap boundary the
        // focus byte is shared by two lines; the selection's
        // `CaretAffinity` disambiguates — Upstream binds the caret to
        // the trailing edge of the earlier line, Downstream (the
        // default) to the leading edge of the later line. Single-line
        // (and every default-affinity caret) → line 0, top 0, and the
        // identical pen x (byte-identical).
        let (top, height, x_offset, caret_x) = caret_line_geometry_affinity(
            shaping,
            &layout,
            content,
            font_size,
            selection.focus(),
            selection.affinity(),
        );
        node_paint.fill_rect(
            Rect::new(
                Point::new(origin.x + x_offset + caret_x - scroll_left, origin.y + top),
                Size::new(CARET_WIDTH, height),
            ),
            Brush::solid(text_color),
        );
    }
}

#[doc = include_str!("../../docs/api/orchestration/text_painter.doc/emit_generated_run.md")]
#[allow(clippy::too_many_arguments)]
pub(super) fn emit_generated_run(
    shaping: &ParagraphShaping,
    node_paint: &mut DisplayListBuilder,
    tree: &RetainedTree,
    computed: &ComputedStyles,
    boxes: &LayoutBoxes,
    node: NodeId,
    origin: Point,
    box_width: f32,
    run: &GeneratedRun,
) {
    let color = run
        .color
        .map_or_else(|| text_color_of(computed.get(&node)), scene_color_from);
    let brush = Brush::solid(color);
    let font_size = font_size_of(computed, node);
    let layout = shaping.shape_text_lines(tree, computed, boxes, node, &run.text, box_width);
    for line in layout.lines() {
        if line.segments().is_empty() {
            continue;
        }
        let baseline = line.baseline();
        let line_top = origin.y + line.top();
        let line_left = origin.x + line.x_offset();
        let mut seg_x = 0.0_f32;
        for segment in line.segments() {
            let range = segment.run().source();
            let seg_text = run.text.get(range.start..range.end).unwrap_or_default();
            let glyph_run = positioned_run(segment, baseline, font_size, seg_text, &run.text);
            node_paint.draw_text(
                Point::new(line_left + seg_x, line_top),
                glyph_run,
                brush.clone(),
            );
            seg_x += segment.run().advance();
        }
    }
}
