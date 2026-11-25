use obscur_text::editing::{EditCommand, EditDirection};
use obscur_text::selection::TextSelection;

#[doc = include_str!("../../docs/api/runtime/key_edit_resolution.doc/KeyEdit.md")]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum KeyEdit {
    /// The key is **not** an edit: it falls through to the shortcut / activation
    /// path byte-identically. Every `Ctrl`/`Alt`/`Meta` chord that is not one of
    /// the editing chords, `Escape`, `Tab`, a vertical arrow in a single-line
    /// field, and a key that produced no logical key at all.
    Declined,
    /// One edit command, to be applied through the runtime's `apply_edit` choke
    /// point (which is where the readonly gate and the post-edit bookkeeping
    /// live). Consumes the key.
    Edit(EditCommand),
    #[doc = include_str!("../../docs/api/runtime/key_edit_resolution.doc/DeleteGrapheme.md")]
    DeleteGrapheme {
        /// Backward for `Backspace`, forward for `Delete`.
        direction: EditDirection,
        /// The multi-scalar cluster to select before deleting, if there is one.
        cluster: Option<TextSelection>,
    },
    /// `Ctrl`/`Meta` + `A`: select the whole buffer. Consumes the key.
    SelectAll,
    /// `Ctrl`/`Meta` + `Z` (`redo = false`), `+Shift+Z` / `+Y` (`redo = true`).
    /// Consumes the key whether or not the buffer had anything to undo or redo.
    History {
        /// Whether the chord asked for a redo rather than an undo.
        redo: bool,
    },
    /// A single-line field's `Enter`: a **commit** (WHATWG HTML §4.10.5.5 /
    /// §4.10.21.3 implicit form submission). It fires `change` with the committed
    /// value and rebaselines the session, inserts nothing, and is **not** consumed
    /// — Enter's routing past the commit is byte-identical.
    Commit,
}
