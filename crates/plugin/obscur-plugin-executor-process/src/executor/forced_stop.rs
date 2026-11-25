#[derive(Clone, Copy)]
pub(super) enum ForcedStop {
    Timeout,
    Quota,
}
