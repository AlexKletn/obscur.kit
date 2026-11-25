use obscur_scene::paint::{SceneImage, SharedPixels};

#[derive(Debug, Clone)]
pub(super) struct CachedMask {
    _source: SharedPixels,
    pub(super) image: SceneImage,
    pub(super) seen: u64,
}

impl CachedMask {
    pub(super) fn new(source: &SceneImage, image: SceneImage, seen: u64) -> Self {
        // Keep the source allocation alive while its address is in the cache key.
        Self {
            _source: source.shared_pixels(),
            image,
            seen,
        }
    }
}
