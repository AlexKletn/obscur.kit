#[derive(Clone, Copy)]
pub(super) struct BitmapInfo {
    pub(super) bytes: u64,
    pub(super) logical_size: (f32, f32),
}
