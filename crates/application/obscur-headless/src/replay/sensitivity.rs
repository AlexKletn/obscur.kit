#[doc = include_str!("../../docs/api/replay/event_recording.doc/Sensitivity.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Sensitivity {
    /// Ordinary, non-sensitive input; recorded verbatim.
    #[default]
    Normal,
    /// A secret (password, token, credential); redacted by default.
    Secret,
    /// Protected clipboard data; redacted by default.
    Clipboard,
}
