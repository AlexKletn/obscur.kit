#![doc = include_str!("../../docs/api/algorithm/grid_track_positions.doc/_module.md")]

use super::content_distribution::content_distribution;
use crate::model::JustifyContent;

#[doc = include_str!("../../docs/api/algorithm/grid_track_positions.doc/track_offsets.md")]
fn track_offsets(sizes: &[f32], gap: f32, origin: f32, collapsed: &[bool]) -> Vec<f32> {
    let mut offsets = Vec::with_capacity(sizes.len());
    let mut cursor = origin;
    for (i, &size) in sizes.iter().enumerate() {
        if collapsed.get(i).copied().unwrap_or(false) {
            // Collapsed to zero width; parks at the cursor without advancing it.
            offsets.push(cursor);
            continue;
        }
        offsets.push(cursor);
        cursor += size + gap;
    }
    offsets
}

#[doc = include_str!("../../docs/api/algorithm/grid_track_positions.doc/distributed_track_offsets.md")]
pub(super) fn distributed_track_offsets(
    sizes: &[f32],
    gap: f32,
    origin: f32,
    collapsed: &[bool],
    content_extent: f32,
    mode: JustifyContent,
) -> Vec<f32> {
    let (n, track_extent) = active_track_extent(sizes, gap, collapsed);
    let leftover = content_extent - track_extent;
    let (leading, extra_gap) = content_distribution(n, leftover, mode);
    // The un-distributed default path stays byte-identical to `track_offsets`.
    if leading == 0.0 && extra_gap == 0.0 {
        return track_offsets(sizes, gap, origin, collapsed);
    }
    let mut offsets = Vec::with_capacity(sizes.len());
    let mut cursor = origin + leading;
    for (i, &size) in sizes.iter().enumerate() {
        if collapsed.get(i).copied().unwrap_or(false) {
            offsets.push(cursor);
            continue;
        }
        offsets.push(cursor);
        cursor += size + gap + extra_gap;
    }
    offsets
}

/// Counts non-collapsed tracks and measures their extent, including internal gaps.
pub(super) fn active_track_extent(sizes: &[f32], gap: f32, collapsed: &[bool]) -> (usize, f32) {
    // Non-collapsed track count and total size, for the leftover computation. A
    // collapsed (empty `auto-fit`) track contributes neither size nor gap.
    let mut n = 0_usize;
    let mut sum = 0.0_f32;
    for (i, &size) in sizes.iter().enumerate() {
        if collapsed.get(i).copied().unwrap_or(false) {
            continue;
        }
        n += 1;
        sum += size;
    }
    (n, sum + gap * n.saturating_sub(1) as f32)
}

/// Marks which tracks of an `auto-fit` repeat region collapse: a track inside the
/// `[start, end)` generated range that no placed item covers. Returns an
/// all-`false` vector (nothing collapses) when there is no `auto-fit` region — so
/// an `auto-fill` or plain template is untouched.
pub(super) fn collapsed_tracks(
    len: usize,
    auto_fit: Option<(usize, usize)>,
    anchors: &[usize],
    spans: &[usize],
) -> Vec<bool> {
    let mut collapsed = vec![false; len];
    let Some((start, end)) = auto_fit else {
        return collapsed;
    };
    let mut used = vec![false; len];
    for (&anchor, &span) in anchors.iter().zip(spans.iter()) {
        for slot in used.iter_mut().take((anchor + span).min(len)).skip(anchor) {
            *slot = true;
        }
    }
    for (track, slot) in collapsed
        .iter_mut()
        .enumerate()
        .take(end.min(len))
        .skip(start)
    {
        if !used[track] {
            *slot = true;
        }
    }
    collapsed
}

/// The pixel extent a `span`-track cell covers: the sum of the spanned track
/// sizes plus the `span - 1` interior gaps between them.
pub(super) fn span_extent(sizes: &[f32], start: usize, span: usize, gap: f32) -> f32 {
    let end = (start + span).min(sizes.len());
    let base: f32 = sizes
        .get(start..end)
        .map_or(0.0, |s| s.iter().copied().sum());
    base + gap * (span.saturating_sub(1) as f32)
}
