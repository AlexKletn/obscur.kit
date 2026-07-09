//! CSS Text 3 `letter-spacing` (§8.1) and `word-spacing` (§8.2): post-shaping
//! advance adjustments applied to positioned runs.
//!
//! Spacing is deliberately modeled as a **pure, width-independent** rewrite of a
//! shaped paragraph's glyph advances, sitting *between* the (cached)
//! [`shape_paragraph`](crate::shaping::ParagraphShaper::shape_paragraph) result
//! and the width-dependent line wrap. Because it is one function, the runtime's
//! measure pass and its compose pass call the *same* seam with the *same* values,
//! so wrapping (which measures the spaced advances) and painting can never drift.
//!
//! Two spec rules, both additive and both deterministic:
//!
//! * **letter-spacing (§8.1)** — "additional spacing (commonly called *tracking*)
//!   between adjacent typographic character units". The resolved length is added
//!   *after* every typographic character unit in the run. A typographic character
//!   unit is a grapheme cluster; lacking a grapheme segmenter here we approximate
//!   it by the shaper's **cluster** grouping (one add per distinct source
//!   cluster), which is exact for the common one-char-per-cluster case and is a
//!   documented refinement for combining sequences. The trailing letter-spacing
//!   at the end of a line is meant to *hang* (be trimmed); this pass adds it
//!   uniformly (after every cluster incl. the last), and the per-line trim — which
//!   §8.1 says is "deferred to the line breaker" — is performed by the wrapper
//!   (`ParagraphLayout::wrap_core` → `hang_line_trailing`), the only stage that
//!   knows where lines end.
//!
//! * **word-spacing (§8.2)** — "additional spacing between 'words'". The resolved
//!   length is added to the advance of each **word separator**. Per the spec the
//!   word-separator set is small; we honor the two that occur in normal text:
//!   U+0020 SPACE and U+00A0 NO-BREAK SPACE. (The Ethiopic/Aegean separators the
//!   spec also lists are a documented omission.)
//!
//! Both lengths may be **negative** (§8.1/§8.2 allow it); the result is *not*
//! clamped here, so a tight negative tracking can produce a small or negative
//! per-glyph advance — the scene and the CPU/Vello backends accumulate advances
//! as a signed pen, so this renders correctly. When both lengths are zero the
//! input runs are returned untouched, byte-for-byte.

use crate::shaping::{GlyphRun, ShapedGlyph, ShapedRun};

/// Returns whether the character at byte offset `cluster` in `text` is a CSS
/// word separator for `word-spacing` (§8.2): U+0020 SPACE or U+00A0 NBSP.
fn is_word_separator(text: &str, cluster: usize) -> bool {
    text.get(cluster..)
        .and_then(|rest| rest.chars().next())
        .is_some_and(|ch| ch == ' ' || ch == '\u{00A0}')
}

/// Applies `letter_spacing` (§8.1) and `word_spacing` (§8.2) to already-shaped
/// `runs` over the source `text`, returning the runs with adjusted advances.
///
/// `letter_spacing` is added after each typographic character unit (approximated
/// by the shaper's cluster grouping); `word_spacing` is added to each word
/// separator's glyph. Glyph identity, cluster mapping, offsets, direction, font,
/// bidi level, and synthesis are all preserved — only `x_advance` changes.
///
/// A zero/zero call is a no-op and returns `runs` unchanged (byte-identical),
/// which is what an absent or `normal` `letter-spacing`/`word-spacing` resolves
/// to, so an unauthored text node is completely unaffected.
#[must_use]
pub fn apply_spacing(
    runs: Vec<ShapedRun>,
    text: &str,
    letter_spacing: f32,
    word_spacing: f32,
) -> Vec<ShapedRun> {
    if letter_spacing == 0.0 && word_spacing == 0.0 {
        return runs;
    }
    runs.into_iter()
        .map(|run| respace_run(&run, text, letter_spacing, word_spacing))
        .collect()
}

/// Rebuilds one [`ShapedRun`] with letter-/word-spacing folded into its advances,
/// carrying every other attribute across unchanged.
fn respace_run(run: &ShapedRun, text: &str, letter_spacing: f32, word_spacing: f32) -> ShapedRun {
    let source = run.run();
    let glyphs = source.glyphs();
    let respaced: Vec<ShapedGlyph> = glyphs
        .iter()
        .enumerate()
        .map(|(i, glyph)| {
            let mut advance = glyph.x_advance();
            // letter-spacing: one add at each cluster *end* — the last glyph whose
            // cluster equals this one (or the final glyph of the run). This adds
            // the tracking once per typographic character unit.
            let cluster_end =
                i + 1 == glyphs.len() || glyphs[i + 1].cluster() != glyph.cluster();
            if cluster_end {
                advance += letter_spacing;
            }
            // word-spacing: one add at each word separator, applied at the
            // cluster *start* so a multi-glyph separator cluster is only widened
            // once.
            let cluster_start = i == 0 || glyphs[i - 1].cluster() != glyph.cluster();
            if cluster_start && is_word_separator(text, glyph.cluster()) {
                advance += word_spacing;
            }
            ShapedGlyph::new(glyph.glyph_id(), glyph.cluster(), advance)
                .with_offset(glyph.x_offset(), glyph.y_offset())
        })
        .collect();
    let respaced_run =
        GlyphRun::new(respaced, source.direction(), source.font_size(), source.source());
    ShapedRun::new(run.font(), respaced_run)
        .with_level(run.level())
        .with_synthesis(run.synth_oblique(), run.synth_bold())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::FontId;
    use crate::shaping::TextDirection;
    use crate::span::TextRange;

    /// One shaped run of one-glyph-per-char over `text`, every glyph 6px wide,
    /// clusters at each char's byte offset (ASCII, so byte == char index).
    fn run_over(text: &str) -> ShapedRun {
        let glyphs: Vec<ShapedGlyph> = text
            .char_indices()
            .enumerate()
            .map(|(gid, (byte, _))| ShapedGlyph::new(gid as u32, byte, 6.0))
            .collect();
        let run = GlyphRun::new(
            glyphs,
            TextDirection::LeftToRight,
            16.0,
            TextRange::new(0, text.len()),
        );
        ShapedRun::new(FontId::PRIMARY, run)
    }

    fn total_advance(runs: &[ShapedRun]) -> f32 {
        runs.iter()
            .flat_map(|r| r.run().glyphs())
            .map(ShapedGlyph::x_advance)
            .sum()
    }

    #[test]
    fn zero_zero_is_identity() {
        let runs = vec![run_over("hello world")];
        let out = apply_spacing(runs.clone(), "hello world", 0.0, 0.0);
        assert_eq!(out, runs, "no-op spacing must return runs byte-for-byte");
    }

    #[test]
    fn letter_spacing_adds_once_per_cluster() {
        // "hello" = 5 clusters × 6px = 30px baseline; +6px letter-spacing per
        // cluster → +30px (one add per typographic character unit, §8.1).
        let baseline = total_advance(&[run_over("hello")]);
        assert_eq!(baseline, 30.0);
        let spaced = apply_spacing(vec![run_over("hello")], "hello", 6.0, 0.0);
        assert_eq!(total_advance(&spaced), 30.0 + 6.0 * 5.0);
    }

    #[test]
    fn word_spacing_adds_per_separator_only() {
        // "ab cd" has exactly one space → +12px once (§8.2), not per character.
        let baseline = total_advance(&[run_over("ab cd")]);
        let spaced = apply_spacing(vec![run_over("ab cd")], "ab cd", 0.0, 12.0);
        assert_eq!(total_advance(&spaced), baseline + 12.0);
    }

    #[test]
    fn nbsp_is_a_word_separator() {
        let text = "a\u{00A0}b";
        let baseline = total_advance(&[run_over(text)]);
        let spaced = apply_spacing(vec![run_over(text)], text, 0.0, 10.0);
        assert_eq!(total_advance(&spaced), baseline + 10.0);
    }

    #[test]
    fn negative_letter_spacing_is_not_clamped() {
        // -2px per cluster on "abc" (3×6=18) → 18 - 6 = 12, allowed to shrink.
        let spaced = apply_spacing(vec![run_over("abc")], "abc", -2.0, 0.0);
        assert_eq!(total_advance(&spaced), 18.0 - 6.0);
    }

    #[test]
    fn preserves_glyph_identity_cluster_and_font() {
        let spaced = apply_spacing(vec![run_over("hi")], "hi", 4.0, 0.0);
        let run = spaced[0].run();
        assert_eq!(spaced[0].font(), FontId::PRIMARY);
        assert_eq!(run.glyph_at(0).unwrap().glyph_id(), 0);
        assert_eq!(run.glyph_at(0).unwrap().cluster(), 0);
        assert_eq!(run.glyph_at(1).unwrap().cluster(), 1);
        assert_eq!(run.source(), TextRange::new(0, 2));
    }
}
