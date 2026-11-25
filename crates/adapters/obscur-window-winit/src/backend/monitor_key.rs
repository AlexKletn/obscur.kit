#[doc = include_str!("../../docs/api/backend/winit_event_loop.doc/MonitorKey.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct MonitorKey {
    /// The monitor name winit reports, or `None` when it reports none.
    pub(super) name: Option<String>,
    /// Position among the same-named monitors of one enumeration.
    pub(super) ordinal: usize,
}
