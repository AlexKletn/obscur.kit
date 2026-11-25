use super::GroupContent;

pub(super) struct CachedTarget {
    pub(super) content: GroupContent<'static>,
    pub(super) revision: u64,
    pub(super) origin: Option<[u32; 2]>,
}
