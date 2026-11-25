#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DamageScope {
    Transform,
    Clip,
    Layer,
}
