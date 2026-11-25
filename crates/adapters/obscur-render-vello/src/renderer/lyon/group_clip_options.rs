#[derive(Clone, Copy)]
pub(super) struct GroupClipOptions {
    pub(super) defer_outer_clip: bool,
    pub(super) scissor: bool,
}

impl Default for GroupClipOptions {
    fn default() -> Self {
        static OPTIONS: std::sync::OnceLock<GroupClipOptions> = std::sync::OnceLock::new();
        *OPTIONS.get_or_init(|| Self {
            defer_outer_clip: std::env::var_os("OBSCUR_LYON_CACHE_OUTER_CLIPS")
                .is_some_and(|value| value == "1"),
            scissor: std::env::var_os("OBSCUR_LYON_CLIP_SCISSOR").is_some_and(|value| value == "1"),
        })
    }
}
