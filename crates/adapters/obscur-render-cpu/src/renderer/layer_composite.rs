//! Pending compositing operation for a CPU layer or image mask.

use super::group_compositing::composite_top_layer;

pub(super) struct LayerComposite {
    pub(super) opacity: f32,
    pub(super) blend: tiny_skia::BlendMode,
    pub(super) color_matrix: Option<[f32; 20]>,
    pub(super) blur: f32,
    pub(super) mask: Option<tiny_skia::Mask>,
}

impl LayerComposite {
    pub(super) fn finish(
        self,
        targets: &mut Vec<tiny_skia::Pixmap>,
        enclosing_clip: Option<&tiny_skia::Mask>,
    ) {
        let clip = self.mask.as_ref().or_else(|| {
            enclosing_clip.filter(|_| {
                self.blur > 0.0
                    || matches!(
                        self.blend,
                        tiny_skia::BlendMode::Source | tiny_skia::BlendMode::Clear
                    )
            })
        });
        composite_top_layer(
            targets,
            self.opacity,
            self.blend,
            self.color_matrix.as_ref(),
            self.blur,
            clip,
        );
    }
}
