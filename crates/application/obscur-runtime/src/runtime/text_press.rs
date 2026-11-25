use obscur_identity::NodeId;
use obscur_text::selection::TextSelection;

#[doc = include_str!("../../docs/api/runtime/text_pointer_selection.doc/TextPress.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextPress {
    /// The press was not on the editable the session covers, or the field has no
    /// laid-out text to resolve a byte offset against. Nothing is selected and no
    /// drag is armed.
    Ignored,
    /// A single click: collapse the selection to a caret at `offset` and arm a
    /// drag-selection anchored there.
    Caret {
        /// The editable the click landed in — the drag's anchor field.
        field: NodeId,
        /// The byte offset the click resolved to.
        offset: usize,
    },
    /// A double (word) or triple (line) click: set this range. **No** drag is
    /// armed — the selection is the whole word or line.
    Range(TextSelection),
}
