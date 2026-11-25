//! Verify that an already rendered opacity group still has the same inputs.

mod content_operation;
mod group_content;
mod group_input;

use content_operation::ContentOperation;
use group_content::GroupContent;
use group_input::GroupInput;

mod draw_content;
use draw_content::DrawContent;

mod composite_content;
use composite_content::CompositeContent;

mod cached_target;
use cached_target::CachedTarget;

use super::group_schedule::GroupSchedule;
use super::mesh::{Draw, Mesh, Operation};
use super::scheduled_group::ScheduledGroup;

#[derive(Default)]
pub(super) struct GroupContentCache {
    slots: Vec<Option<CachedTarget>>,
    revision: u64,
    root_prefix: Option<GroupContent<'static>>,
}

impl GroupContentCache {
    pub(super) fn enabled() -> bool {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| {
            std::env::var_os("OBSCUR_LYON_CACHE_GROUPS").is_some_and(|value| value == "1")
        })
    }

    pub(super) fn clear(&mut self) {
        self.slots.clear();
        self.root_prefix = None;
    }

    pub(super) fn invalidate(&mut self, index: usize) {
        if let Some(slot) = self.slots.get_mut(index) {
            *slot = None;
        }
    }

    pub(super) fn root_prefix(
        &mut self,
        mesh: &Mesh,
        transforms: &[[[f32; 4]; 2]],
        end: usize,
    ) -> Option<(usize, bool)> {
        let draws = &mesh.draws[..end];
        let end = draws.iter().rposition(|draw| {
            matches!(
                draw.operation,
                Operation::Paint | Operation::Image(_) | Operation::Pop
            )
        })? + 1;
        if !draws[..end]
            .iter()
            .any(|draw| matches!(draw.operation, Operation::Paint | Operation::Image(_)))
        {
            return None;
        }
        let (content, _) = GroupContent::from_inputs(
            draws[..end].iter().map(GroupInput::Draw).collect(),
            mesh,
            transforms,
            None,
        )?;
        let hit = self.root_prefix.as_ref() == Some(&content);
        if !hit {
            self.root_prefix = Some(content.into_owned());
        }
        Some((end, hit))
    }

    pub(super) fn update(
        &mut self,
        schedule: &GroupSchedule,
        mesh: &Mesh,
        transforms: &[[[f32; 4]; 2]],
        extent: (u32, u32),
    ) -> Vec<Option<[i32; 2]>> {
        self.slots.resize_with(schedule.groups.len(), || None);
        let mut hits = vec![None; schedule.groups.len()];
        for (index, group) in schedule.groups.iter().enumerate().rev() {
            let bounds = translation_bounds(group, extent);
            let next = self
                .inputs(group, schedule, &mesh.draws)
                .and_then(|inputs| GroupContent::from_inputs(inputs, mesh, transforms, bounds));
            let slot = &mut self.slots[index];
            if let Some((content, origin)) = next {
                let shift = slot
                    .as_ref()
                    .filter(|entry| entry.content == content)
                    .and_then(|entry| translation_delta(entry.origin, origin));
                if shift.is_some() {
                    hits[index] = shift;
                } else {
                    self.revision += 1;
                    *slot = Some(CachedTarget {
                        content: content.into_owned(),
                        revision: self.revision,
                        origin,
                    });
                }
            } else {
                *slot = None;
            }
        }
        hits
    }

    fn inputs<'a>(
        &self,
        group: &'a ScheduledGroup,
        schedule: &'a GroupSchedule,
        draws: &'a [Draw],
    ) -> Option<Vec<GroupInput<'a>>> {
        let mut inputs = Vec::new();
        if group.composite_clip().is_none() {
            inputs.extend(
                group
                    .clips
                    .iter()
                    .map(|&index| GroupInput::Draw(&draws[index])),
            );
        }
        let mut start = group.draws.start;
        for &index in &group.children {
            let child = &schedule.groups[index];
            inputs.extend(
                draws[start..child.draws.start - 1]
                    .iter()
                    .map(GroupInput::Draw),
            );
            if child.bounds[2] != 0 && child.bounds[3] != 0 && child.opacity != 0.0 {
                inputs.push(GroupInput::Composite(CompositeContent {
                    revision: self.slots[index].as_ref()?.revision,
                    opacity: child.opacity.to_bits(),
                    bounds: child.bounds,
                    stencil_depth: child.composite_clip(),
                }));
            }
            inputs.extend(child.popped_clips.iter().map(GroupInput::Draw));
            start = child.draws.end + 1;
        }
        inputs.extend(draws[start..group.draws.end].iter().map(GroupInput::Draw));
        Some(inputs)
    }
}

fn translation_bounds(group: &ScheduledGroup, extent: (u32, u32)) -> Option<[u32; 4]> {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let enabled = *ENABLED.get_or_init(|| {
        std::env::var_os("OBSCUR_LYON_CACHE_TRANSLATIONS").is_some_and(|value| value == "1")
    });
    let [x, y, w, h] = group.bounds;
    (enabled
        && group.children.is_empty()
        && (group.clips.is_empty() || group.composite_clip().is_some())
        && x > 1
        && y > 1
        && w > 0
        && h > 0
        && x + w + 1 < extent.0
        && y + h + 1 < extent.1)
        .then_some(group.bounds)
}

fn translation_delta(previous: Option<[u32; 2]>, current: Option<[u32; 2]>) -> Option<[i32; 2]> {
    match (previous, current) {
        (None, None) => Some([0, 0]),
        (Some([px, py]), Some([x, y])) => {
            let x = i64::from(x) - i64::from(px);
            let y = i64::from(y) - i64::from(py);
            ((-8192..8192).contains(&x) && (-8192..8192).contains(&y)).then(|| {
                [
                    i32::try_from(x).expect("bounded x"),
                    i32::try_from(y).expect("bounded y"),
                ]
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn cached_draws_use_indices_relative_to_their_vertices() {
        let draw = |base_vertex, indices| DrawContent {
            operation: Operation::Paint,
            depth: 0,
            image: None,
            base_vertex,
            indices: Cow::Borrowed(indices),
            vertices: Cow::Borrowed(&[1, 2, 3]),
        };
        let cached = draw(0, &[0, 2, 1]);
        assert!(cached == draw(8, &[8, 10, 9]));
        assert!(cached != draw(8, &[8, 9, 10]));
        assert!(cached != draw(8, &[8, 10]));
    }
}
