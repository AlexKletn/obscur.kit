use obscur_scene::clip::Clip;
use vello::kurbo::Affine;

/// A pending Vello layer or clip and the state needed to close it.
pub(super) struct SceneScope<'a> {
    pub(super) clip: Option<(&'a Clip, Affine)>,
    /// Transform depth outside a layer. A clip leaves the transform stack alone.
    pub(super) layer_transform_depth: Option<usize>,
}
