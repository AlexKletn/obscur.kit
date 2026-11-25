use std::ops::Range;

use super::mesh::Draw;

pub(super) struct ScheduledGroup {
    pub(super) draws: Range<usize>,
    pub(super) children: Vec<usize>,
    pub(super) clips: Vec<usize>,
    pub(super) popped_clips: Vec<Draw>,
    pub(super) bounds: [u32; 4],
    pub(super) opacity: f32,
    pub(super) defer_outer_clip: bool,
}

impl ScheduledGroup {
    pub(super) fn composite_clip(&self) -> Option<u32> {
        if self.defer_outer_clip
            && self.children.is_empty()
            && !self.clips.is_empty()
            && self.popped_clips.is_empty()
        {
            Some(u32::try_from(self.clips.len()).expect("bounded stencil depth"))
        } else {
            None
        }
    }
}
