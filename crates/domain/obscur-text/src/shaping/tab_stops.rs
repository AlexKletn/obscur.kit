//! CSS Text 3 `tab-size` (§4.3): a preserved tab (U+0009) advances the pen to
//! the next tab stop.
//!
//! Like [`apply_spacing`](crate::shaping::apply_spacing), this is a **pure,
//! width-independent** rewrite of a shaped paragraph's glyph advances that sits
//! between the (cached) shaping result and the width-dependent line wrap, so the
//! runtime's measure pass and its compose pass call the same seam with the same
//! `tab_width` and can never drift.
//!
//! A tab stop is a multiple of the `tab_width` measured from the **start of the
//! line**. Per §4.3 the pen advances to the next tab stop that is *strictly*
//! greater than the current position, so a tab always advances at least a little:
//!
//! ```text
//! next = (floor(pen / tab_width) + 1) · tab_width
//! advance = next − pen
//! ```
//!
//! (equivalently `ceil((pen + ε) / tab_width) · tab_width`). The pen is measured
//! from the line start, so it is reset to zero after each preserved segment break
//! (`\n`) — a documented single-line-per-segment approximation that is exact for
//! the common case where a run of preserved text has no leading indentation.
//!
//! Only tabs that are still present in `text` are affected; under a collapsing
//! `white-space` mode (`normal` / `nowrap` / `pre-line`) a tab has already become
//! a space at the pre-shape seam, so `text` contains no `\t` and this pass is a
//! byte-for-byte no-op (it early-returns before touching any run).

use crate::shaping::{GlyphRun, ShapedGlyph, ShapedRun};

/// Returns the character at byte offset `cluster` in `text`, if any.
fn char_at(text: &str, cluster: usize) -> Option<char> {
    text.get(cluster..).and_then(|rest| rest.chars().next())
}

/// Advances every preserved tab (U+0009) in already-shaped `runs` over the source
/// `text` to the next `tab_width` tab stop (§4.3), returning the runs with the tab
/// glyphs' advances rewritten.
///
/// `tab_width` is the resolved tab-stop width in logical pixels (`tab-size` as a
/// `<number>` times the space advance, or a `<length>` verbatim). Runs are walked
/// in logical order with a running pen measured from the line start; the pen is
/// reset after each preserved `\n`. Only the advance of a tab glyph changes —
/// glyph identity, cluster, offsets, font, direction, level, and synthesis are
/// preserved.
///
/// The pass early-returns `runs` unchanged when `tab_width <= 0` or `text`
/// contains no tab, so an unauthored or collapsing-mode text node is completely
/// unaffected (byte-for-byte identical).
#[must_use]
pub fn apply_tab_stops(runs: Vec<ShapedRun>, text: &str, tab_width: f32) -> Vec<ShapedRun> {
    if tab_width <= 0.0 || !text.contains('\t') {
        return runs;
    }
    let mut pen = 0.0_f32;
    runs.into_iter()
        .map(|run| retab_run(&run, text, tab_width, &mut pen))
        .collect()
}

/// Rebuilds one [`ShapedRun`], advancing its tab glyphs to the next stop and
/// carrying the running `pen` across the run (resetting it after each `\n`).
fn retab_run(run: &ShapedRun, text: &str, tab_width: f32, pen: &mut f32) -> ShapedRun {
    let source = run.run();
    let retabbed: Vec<ShapedGlyph> = source
        .glyphs()
        .iter()
        .map(|glyph| {
            let ch = char_at(text, glyph.cluster());
            let advance = if ch == Some('\t') {
                // Advance to the next tab stop strictly past the current pen.
                let stops = (*pen / tab_width).floor() + 1.0;
                (stops * tab_width - *pen).max(0.0)
            } else {
                glyph.x_advance()
            };
            let out = ShapedGlyph::new(glyph.glyph_id(), glyph.cluster(), advance)
                .with_offset(glyph.x_offset(), glyph.y_offset());
            if ch == Some('\n') {
                // A preserved segment break starts a new line: reset the pen.
                *pen = 0.0;
            } else {
                *pen += advance;
            }
            out
        })
        .collect();
    let retabbed_run =
        GlyphRun::new(retabbed, source.direction(), source.font_size(), source.source());
    ShapedRun::new(run.font(), retabbed_run)
        .with_level(run.level())
        .with_synthesis(run.synth_oblique(), run.synth_bold())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::FontId;
    use crate::shaping::TextDirection;
    use crate::span::TextRange;

    /// One shaped run of one-glyph-per-char over `text`, each glyph `advance` px
    /// wide, clusters at each char's byte offset (ASCII, so byte == char index).
    fn run_over(text: &str, advance: f32) -> ShapedRun {
        let glyphs: Vec<ShapedGlyph> = text
            .char_indices()
            .enumerate()
            .map(|(gid, (byte, _))| ShapedGlyph::new(gid as u32, byte, advance))
            .collect();
        let run = GlyphRun::new(
            glyphs,
            TextDirection::LeftToRight,
            16.0,
            TextRange::new(0, text.len()),
        );
        ShapedRun::new(FontId::PRIMARY, run)
    }

    fn advances(runs: &[ShapedRun]) -> Vec<f32> {
        runs.iter()
            .flat_map(|r| r.run().glyphs())
            .map(ShapedGlyph::x_advance)
            .collect()
    }

    #[test]
    fn no_tab_is_identity() {
        let runs = vec![run_over("abc", 6.0)];
        let out = apply_tab_stops(runs.clone(), "abc", 40.0);
        assert_eq!(out, runs, "text with no tab must return runs byte-for-byte");
    }

    #[test]
    fn zero_width_is_identity() {
        let runs = vec![run_over("a\tb", 6.0)];
        let out = apply_tab_stops(runs.clone(), "a\tb", 0.0);
        assert_eq!(out, runs);
    }

    #[test]
    fn tab_advances_to_next_stop() {
        // "a\tb", pen after "a" = 6; tab_width 40 → next stop 40, so the tab's
        // advance becomes 40 − 6 = 34. "b" keeps its 6.
        let out = apply_tab_stops(vec![run_over("a\tb", 6.0)], "a\tb", 40.0);
        assert_eq!(advances(&out), vec![6.0, 34.0, 6.0]);
    }

    #[test]
    fn tab_on_a_stop_advances_to_the_next_one() {
        // Pen exactly on a stop (0) still advances a full tab_width (a tab is never
        // zero-width): "\t" at pen 0 with width 32 → 32.
        let out = apply_tab_stops(vec![run_over("\tx", 8.0)], "\tx", 32.0);
        assert_eq!(advances(&out), vec![32.0, 8.0]);
    }

    #[test]
    fn pen_resets_after_a_preserved_newline() {
        // "aa\n\tb": pen after "aa" = 20, `\n` resets pen to 0, so the tab at the
        // new line start advances a full stop (32), not 32 − 20.
        let out = apply_tab_stops(vec![run_over("aa\n\tb", 10.0)], "aa\n\tb", 32.0);
        // glyphs: a(10) a(10) \n(10, unchanged) \t(→32 from pen 0) b(10)
        assert_eq!(advances(&out), vec![10.0, 10.0, 10.0, 32.0, 10.0]);
    }
}
