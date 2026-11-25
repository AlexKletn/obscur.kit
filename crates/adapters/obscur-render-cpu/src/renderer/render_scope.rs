use super::layer_composite::LayerComposite;

/// State pushed by a clip or layer, including the disposition of any offscreen target.
pub(super) struct RenderScope {
    pub(super) clip: Option<tiny_skia::Mask>,
    pub(super) composite: Option<LayerComposite>,
    /// Transform stack depth before a layer pushed its transform. Clips push none.
    pub(super) layer_transform_depth: Option<usize>,
}

impl RenderScope {
    pub(super) fn clip(&self) -> Option<&tiny_skia::Mask> {
        self.clip.as_ref()
    }

    pub(super) fn finish(
        self,
        transforms: &mut Vec<tiny_skia::Transform>,
        targets: &mut Vec<tiny_skia::Pixmap>,
        enclosing_clip: Option<&tiny_skia::Mask>,
    ) {
        if let Some(depth) = self.layer_transform_depth {
            transforms.truncate(depth);
        }
        if let Some(composite) = self.composite {
            composite.finish(targets, enclosing_clip);
        }
    }
}
