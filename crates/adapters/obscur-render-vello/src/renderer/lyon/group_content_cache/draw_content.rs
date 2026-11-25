use std::borrow::Cow;

use obscur_scene::paint::SceneImage;

use super::Operation;

pub(super) struct DrawContent<'a> {
    pub(super) operation: Operation,
    pub(super) depth: u32,
    pub(super) image: Option<SceneImage>,
    pub(super) base_vertex: u32,
    pub(super) indices: Cow<'a, [u32]>,
    pub(super) vertices: Cow<'a, [u8]>,
}

impl PartialEq for DrawContent<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.operation == other.operation
            && self.depth == other.depth
            && self.image == other.image
            && self.indices.len() == other.indices.len()
            && self.vertices == other.vertices
            && if self.base_vertex == other.base_vertex {
                self.indices == other.indices
            } else {
                self.indices
                    .iter()
                    .zip(other.indices.iter())
                    .all(|(left, right)| left - self.base_vertex == right - other.base_vertex)
            }
    }
}
