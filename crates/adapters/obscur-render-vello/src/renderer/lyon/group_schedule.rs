//! Schedule independent opacity groups before the root render pass.

use super::group_clip_options::GroupClipOptions;
use super::scheduled_group::ScheduledGroup;

use super::mesh::{Draw, Mesh, Operation, pixel_bounds};
use super::pipelines::SAMPLES;

const MAX_GROUPS: usize = 6;
const TARGET_BUDGET_BYTES: u64 = 128 * 1024 * 1024;

pub(super) struct GroupSchedule {
    pub(super) roots: Vec<usize>,
    pub(super) groups: Vec<ScheduledGroup>,
}

pub(super) fn root_cache_enabled(groups: usize, extent: (u32, u32)) -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("OBSCUR_LYON_CACHE_ROOT").is_some_and(|v| v == "1"))
        && fits_target_budget(groups, extent)
}

// Include the target about to be added. Overflow implies the total exceeds
// the budget, so the caller can fall back to the regular pass.
fn fits_target_budget(groups: usize, (width, height): (u32, u32)) -> bool {
    u64::try_from(groups)
        .ok()
        .and_then(|groups| groups.checked_add(1))
        .and_then(|targets| targets.checked_mul(u64::from(width) * u64::from(height)))
        .and_then(|pixels| pixels.checked_mul(u64::from(SAMPLES) * 4))
        .is_some_and(|bytes| bytes <= TARGET_BUDGET_BYTES)
}

fn clip_bounds(mesh: &Mesh, draw: &Draw, extent: (u32, u32)) -> Option<[u32; 4]> {
    let indices = mesh
        .indices
        .get(draw.indices.start as usize..draw.indices.end as usize)?;
    let first = *indices.iter().min()? as usize;
    let last = *indices.iter().max()? as usize;
    Some(pixel_bounds(
        mesh.vertices.get(first..=last)?,
        &mesh.parameters,
        extent,
    ))
}

fn intersect([x, y, w, h]: [u32; 4], [a, b, c, d]: [u32; 4]) -> [u32; 4] {
    let left = x.max(a);
    let top = y.max(b);
    // Measure from the shared origin so large far edges need no addition.
    [
        left,
        top,
        w.saturating_sub(left - x).min(c.saturating_sub(left - a)),
        h.saturating_sub(top - y).min(d.saturating_sub(top - b)),
    ]
}

impl GroupSchedule {
    pub(super) fn clip_scissors(&self, mesh: &Mesh, extent: (u32, u32)) -> Vec<[u32; 4]> {
        let mut clips = std::collections::HashMap::new();
        self.groups
            .iter()
            .map(|group| {
                if group.composite_clip().is_none() {
                    return group.bounds;
                }
                group.clips.iter().fold(group.bounds, |bounds, &index| {
                    let clip = clips
                        .entry(index)
                        .or_insert_with(|| clip_bounds(mesh, &mesh.draws[index], extent));
                    clip.map_or(bounds, |clip| intersect(bounds, clip))
                })
            })
            .collect()
    }

    pub(super) fn for_draws(
        draws: &[Draw],
        extent: (u32, u32),
        options: GroupClipOptions,
    ) -> Option<Self> {
        let mut schedule = Self {
            roots: Vec::new(),
            groups: Vec::<ScheduledGroup>::new(),
        };
        let mut parents = Vec::<usize>::new();
        let mut clips = Vec::<usize>::new();
        for (index, draw) in draws.iter().enumerate() {
            match draw.operation {
                Operation::BeginGroup => {
                    let group = schedule.groups.len();
                    if group == MAX_GROUPS || !fits_target_budget(group, extent) {
                        return None;
                    }
                    if let Some(&parent) = parents.last() {
                        schedule.groups.get_mut(parent)?.children.push(group);
                    } else {
                        schedule.roots.push(group);
                    }
                    schedule.groups.push(ScheduledGroup {
                        draws: index + 1..index + 1,
                        children: Vec::new(),
                        clips: clips.clone(),
                        popped_clips: Vec::new(),
                        bounds: [0; 4],
                        opacity: 0.0,
                        defer_outer_clip: options.defer_outer_clip,
                    });
                    parents.push(group);
                }
                Operation::EndGroup {
                    encode_srgb: false,
                    blend: None,
                    opacity,
                    bounds,
                    color_filter: None,
                    blur_radius: 0.0,
                    stencil_depth,
                } => {
                    let group = &mut schedule.groups[parents.pop()?];
                    if !group.clips.starts_with(&clips) || stencil_depth as usize != clips.len() {
                        return None;
                    }
                    // A clip on the group layer itself is pushed before BeginGroup
                    // and popped before EndGroup. Propagate that pop to the parent.
                    group
                        .popped_clips
                        .extend(group.clips[clips.len()..].iter().rev().map(|&index| {
                            let push = &draws[index];
                            Draw {
                                operation: Operation::Pop,
                                depth: push.depth + 1,
                                indices: push.indices.clone(),
                            }
                        }));
                    group.draws.end = index;
                    group.bounds = bounds;
                    group.opacity = opacity;
                }
                Operation::Push if draw.depth as usize == clips.len() => clips.push(index),
                Operation::Pop if draw.depth as usize == clips.len() => {
                    let push = &draws[clips.pop()?];
                    if push.indices != draw.indices {
                        return None;
                    }
                }
                Operation::Paint | Operation::Image(_) => {}
                _ => return None,
            }
        }
        (parents.is_empty() && clips.is_empty() && schedule.groups.len() >= 2).then_some(schedule)
    }
}

#[cfg(test)]
mod tests {
    use super::{fits_target_budget, intersect};

    #[test]
    fn clip_intersection_handles_far_edges_above_u32_range() {
        assert_eq!(
            intersect([u32::MAX - 2, 0, 8, 10], [u32::MAX, 4, 9, 10]),
            [u32::MAX, 4, 6, 6]
        );
    }

    #[test]
    fn rejects_extents_whose_target_bytes_would_overflow() {
        assert!(fits_target_budget(1, (1024, 768)));
        assert!(!fits_target_budget(1, (u32::MAX, u32::MAX)));
        assert!(!fits_target_budget(usize::MAX, (1024, 768)));
    }
}
