//! Apply image coverage to an isolated group through the existing compositor.

use obscur_geometry::{AffineTransform, Rect};
use obscur_scene::clip::Clip;
use obscur_scene::paint::SceneImage;
use vello::peniko::Compose;

use super::super::Operation;
use super::{MeshEmitter, PaintGroup};

pub(super) struct ImageMask {
    image: SceneImage,
    bounds: Rect,
    luminance: bool,
    transform: AffineTransform,
    slot: u32,
}

impl ImageMask {
    pub(super) fn from_clip(clip: &Clip, transform: AffineTransform, slot: u32) -> Option<Self> {
        let Clip::ImageMask {
            kind,
            image,
            bounds,
        } = clip
        else {
            return None;
        };
        Some(Self {
            image: image.clone(),
            bounds: *bounds,
            luminance: kind.kind().is_luminance(),
            transform,
            slot,
        })
    }
}

impl MeshEmitter<'_> {
    pub(super) fn open_mask(&mut self, mask: ImageMask, groups: &mut usize) -> PaintGroup {
        *groups += 1;
        self.mesh.group_depth = self.mesh.group_depth.max(*groups);
        self.mesh.group_marker(Operation::BeginGroup);
        PaintGroup {
            mask: Some(mask),
            blend: None,
            replacement_bounds: None,
            opacity: 1.0,
            vertex_start: self.mesh.vertices.len(),
            color_filter: None,
            blur_radius: 0.0,
            preceding_blurs: self.mesh.blur_count,
        }
    }

    pub(super) fn apply_mask(
        &mut self,
        mask: ImageMask,
        bounds: [u32; 4],
        depth: u32,
        groups: usize,
    ) -> Result<(), &'static str> {
        self.mesh.needs_blend = true;
        self.mesh.group_depth = self.mesh.group_depth.max(groups + 1);
        self.mesh.group_marker(Operation::BeginGroup);
        let first = self.mesh.vertices.len();
        let previous = std::mem::replace(&mut self.transform_slot, mask.slot);
        let draw = self.image(
            &mask.image,
            mask.bounds,
            mask.bounds,
            [false; 2],
            mask.transform,
            0.0,
        );
        self.transform_slot = previous;
        if let Some((image, indices)) = draw? {
            // The image shader interprets a zero red lane as a luminance mask.
            if mask.luminance {
                for vertex in &mut self.mesh.vertices[first..] {
                    vertex.color[0] = 0.0;
                }
            }
            self.mesh.draw(Operation::Image(image), depth, indices);
        }
        self.mesh.group_marker(Operation::EndGroup {
            encode_srgb: false,
            blend: Some(Compose::DestIn as u32),
            opacity: 1.0,
            bounds,
            color_filter: None,
            blur_radius: 0.0,
            stencil_depth: depth,
        });
        Ok(())
    }
}
