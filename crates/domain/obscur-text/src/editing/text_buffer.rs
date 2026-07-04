//! The editing engine: a pure, deterministic text + selection state machine
//! (spec §5.11 editing engine).
//!
//! [`TextBuffer`] owns a UTF-8 [`String`], a single [`TextSelection`], and the
//! local undo/redo stacks. It applies [`EditCommand`]s — resolving navigation,
//! deletion, insertion and replacement against the current text and selection —
//! and records each text change as a coalescable [`UndoUnit`] of invertible
//! [`EditTransaction`]s. Everything here is deterministic and layout-free:
//! `TextInput`, `TextArea`, and higher-level editors are built on this engine.
//!
//! # UTF-8 correctness
//!
//! All offsets are *byte* offsets that this engine keeps on UTF-8 character
//! boundaries. Untrusted offsets (from [`set_selection`](TextBuffer::set_selection)
//! or [`EditCommand::Replace`]) are clamped into range and snapped down to the
//! nearest character boundary before any slicing; the buffer never slices on a
//! non-boundary.
//!
//! # Simplifications (adapter concerns)
//!
//! This crate is pure-domain and has no text layout, so several motions are
//! approximated and are documented on their resolvers:
//!
//! * **Grapheme** motion treats one `char` (Unicode scalar value) as one
//!   grapheme. True extended grapheme clusters (combining marks, ZWJ emoji,
//!   regional indicators) are a shaping/adapter concern; a layout-aware adapter
//!   can pre-resolve cluster boundaries and drive the buffer with explicit
//!   ranges via [`EditCommand::Replace`] / [`EditCommand::SetSelection`].
//! * **Word** motion is a simplified UAX-29 approximation: skip a run of
//!   whitespace then a run of non-whitespace in the travel direction.
//! * **Line** and **Paragraph** motion collapse to *logical* line boundaries
//!   (delimited by `'\n'`); true vertical/visual up-down motion needs layout and
//!   is left to an adapter.

use crate::selection::TextSelection;
use crate::span::TextRange;

use super::{
    DeletionUnit, EditCommand, EditDirection, EditTransaction, NavigationUnit, UndoKind, UndoUnit,
};

/// The pure editing engine: UTF-8 text, one selection, and local undo/redo
/// (spec §5.11 editing engine).
///
/// A `TextBuffer` is a deterministic state machine. [`apply`](Self::apply)
/// consumes an [`EditCommand`] and updates the text and selection, pushing an
/// [`UndoUnit`] for every command that actually changes text; [`undo`](Self::undo)
/// and [`redo`](Self::redo) walk those units, restoring both the text and the
/// recorded selection. Runs of typing and of deletion coalesce into a single
/// undo step (see [`UndoKind`]); every other kind of edit is atomic.
///
/// The buffer holds exactly one [`TextSelection`]. Multi-selection is a
/// higher-level concern layered on top of this engine.
#[derive(Debug, Clone, PartialEq)]
pub struct TextBuffer {
    /// The UTF-8 buffer contents.
    text: String,
    /// The current selection (collapsed selections behave as a plain caret).
    selection: TextSelection,
    /// Undo stack; the top is the most recent unit and the next to be undone.
    undo: Vec<UndoUnit>,
    /// Redo stack; the top is the most recently undone unit.
    redo: Vec<UndoUnit>,
}

impl TextBuffer {
    /// Creates an empty buffer with a collapsed selection (caret) at offset `0`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            text: String::new(),
            selection: TextSelection::caret(0),
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    /// Creates a buffer seeded with `text` and a collapsed selection at offset
    /// `0`. The undo/redo history starts empty.
    #[must_use]
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            selection: TextSelection::caret(0),
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    /// Returns the current buffer contents.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the current selection.
    #[must_use]
    pub const fn selection(&self) -> TextSelection {
        self.selection
    }

    /// Returns the length of the buffer in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Returns whether the buffer contains no text.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns whether there is at least one undo step available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Returns whether there is at least one redo step available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Replaces the current selection with `selection`, clamping both endpoints
    /// into `0..=len` and snapping each to the nearest character boundary. The
    /// focus affinity is preserved.
    pub fn set_selection(&mut self, selection: TextSelection) {
        let anchor = self.snap_to_boundary(selection.anchor());
        let focus = self.snap_to_boundary(selection.focus());
        self.selection = TextSelection::new(anchor, focus).with_affinity(selection.affinity());
    }

    /// Applies one editing command, updating the text and selection
    /// deterministically.
    ///
    /// Mutating commands ([`Insert`](EditCommand::Insert),
    /// [`Delete`](EditCommand::Delete), [`Replace`](EditCommand::Replace)) push
    /// an [`UndoUnit`] and clear the redo stack; navigation commands
    /// ([`Move`](EditCommand::Move), [`SelectAll`](EditCommand::SelectAll),
    /// [`SetSelection`](EditCommand::SetSelection)) only move the selection and
    /// leave the undo/redo stacks untouched.
    pub fn apply(&mut self, command: EditCommand) {
        match command {
            EditCommand::Insert(text) => {
                self.replace_range(self.selection.range(), &text, UndoKind::Typing);
            }
            EditCommand::Replace { range, text } => {
                self.replace_range(range, &text, UndoKind::Other);
            }
            EditCommand::Delete { unit, direction } => {
                let range = if self.selection.is_collapsed() {
                    self.resolve_deletion(self.selection.focus(), unit, direction)
                } else {
                    self.selection.range()
                };
                if range.is_empty() {
                    return;
                }
                self.replace_range(range, "", UndoKind::Deletion);
            }
            EditCommand::Move {
                unit,
                direction,
                extend,
            } => {
                let target = self.resolve_navigation(self.selection.focus(), unit, direction);
                self.selection = if extend {
                    self.selection.extend_to(target)
                } else {
                    TextSelection::caret(target)
                };
            }
            EditCommand::SelectAll => {
                self.selection = TextSelection::new(0, self.text.len());
            }
            EditCommand::SetSelection(selection) => self.set_selection(selection),
        }
    }

    /// Undoes the most recent undo unit, restoring the text and the selection
    /// that preceded it, and returns whether a step was undone.
    pub fn undo(&mut self) -> bool {
        let Some(unit) = self.undo.pop() else {
            return false;
        };
        // `invert` already reverses order and inverts each edit, so applying the
        // inverse edits in sequence restores the pre-unit text.
        let inverse = unit.invert();
        self.apply_edits_raw(inverse.edits());
        if let Some(selection) = unit.selection_before() {
            self.selection = selection;
        }
        self.redo.push(unit);
        true
    }

    /// Re-applies the most recently undone unit, restoring the text and the
    /// selection that followed it, and returns whether a step was redone.
    pub fn redo(&mut self) -> bool {
        let Some(unit) = self.redo.pop() else {
            return false;
        };
        self.apply_edits_raw(unit.edits());
        if let Some(selection) = unit.selection_after() {
            self.selection = selection;
        }
        self.undo.push(unit);
        true
    }

    // -- private helpers ----------------------------------------------------

    /// Snaps `offset` into `0..=len` and down to the nearest character boundary
    /// at or below it, so it is always safe to slice on.
    fn snap_to_boundary(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.text.len());
        while offset > 0 && !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    /// Splices `new_text` into `range`, records the invertible transaction (with
    /// coalescing per `kind`), and moves the caret to the end of the inserted
    /// text. A no-op when the replacement leaves the bytes unchanged.
    fn replace_range(&mut self, range: TextRange, new_text: &str, kind: UndoKind) {
        // Snap both bounds to character boundaries and re-normalize.
        let a = self.snap_to_boundary(range.start);
        let b = self.snap_to_boundary(range.end);
        let (start, end) = if a <= b { (a, b) } else { (b, a) };
        let range = TextRange::new(start, end);

        let removed = self.text[start..end].to_owned();
        if removed == new_text {
            // Nothing actually changes; do not disturb the history.
            return;
        }

        let selection_before = self.selection;
        self.text.replace_range(start..end, new_text);

        let new_caret = start + new_text.len();
        let selection_after = TextSelection::caret(new_caret);
        self.selection = selection_after;

        let edit =
            EditTransaction::new(range, removed, new_text, selection_before, selection_after);

        let coalesce = kind.is_coalescing()
            && self.undo.last().is_some_and(|unit| {
                unit.can_coalesce_with(kind) && Self::is_contiguous(unit, &edit, kind)
            });

        if coalesce {
            // `coalesce` implies the stack is non-empty.
            if let Some(unit) = self.undo.last_mut() {
                unit.push(edit);
            }
        } else {
            self.undo.push(UndoUnit::single(edit, kind));
        }

        self.redo.clear();
    }

    /// Returns whether `edit` is contiguous with the last edit of `unit` for the
    /// purposes of coalescing a `kind` run.
    ///
    /// * Typing coalesces only when the new insertion begins exactly where the
    ///   previous insertion ended.
    /// * Deletion coalesces when the new range is adjacent to the previous one
    ///   on either side: the backspace case (new end meets previous start) and
    ///   the forward-delete case (new start equals previous start, since forward
    ///   deletion keeps eating text at the same offset).
    fn is_contiguous(unit: &UndoUnit, edit: &EditTransaction, kind: UndoKind) -> bool {
        let Some(last) = unit.edits().last() else {
            return false;
        };
        match kind {
            UndoKind::Typing => edit.range().start == last.inserted_range().end,
            UndoKind::Deletion => {
                edit.range().end == last.range().start || edit.range().start == last.range().start
            }
            UndoKind::Composition | UndoKind::Paste | UndoKind::Other => false,
        }
    }

    /// Splices each edit in order without generating new transactions — used by
    /// [`undo`](Self::undo) and [`redo`](Self::redo). Each edit's range is in the
    /// coordinate space produced by the edits before it, so they apply in order.
    fn apply_edits_raw(&mut self, edits: &[EditTransaction]) {
        for edit in edits {
            let range = edit.range();
            self.text
                .replace_range(range.start..range.end, edit.inserted());
        }
    }

    /// Resolves the target caret offset for a navigation command from `offset`.
    /// See the module-level notes on the Grapheme/Word/Line simplifications.
    fn resolve_navigation(
        &self,
        offset: usize,
        unit: NavigationUnit,
        direction: EditDirection,
    ) -> usize {
        let offset = self.snap_to_boundary(offset);
        match unit {
            // One `char` approximates one grapheme (adapter concern).
            NavigationUnit::Grapheme => match direction {
                EditDirection::Backward => self.prev_boundary(offset),
                EditDirection::Forward => self.next_boundary(offset),
            },
            // Simplified UAX-29 word motion (adapter concern).
            NavigationUnit::Word => match direction {
                EditDirection::Backward => self.prev_word(offset),
                EditDirection::Forward => self.next_word(offset),
            },
            // No layout: Line/Paragraph collapse to logical line boundaries.
            NavigationUnit::Line | NavigationUnit::Paragraph | NavigationUnit::LineBoundary => {
                match direction {
                    EditDirection::Backward => self.line_start(offset),
                    EditDirection::Forward => self.line_end(offset),
                }
            }
            NavigationUnit::Document => match direction {
                EditDirection::Backward => 0,
                EditDirection::Forward => self.text.len(),
            },
        }
    }

    /// Resolves the byte range a deletion command removes when starting from a
    /// collapsed caret at `offset`. Returns an empty range when there is nothing
    /// to delete (e.g. Backspace at offset `0`).
    fn resolve_deletion(
        &self,
        offset: usize,
        unit: DeletionUnit,
        direction: EditDirection,
    ) -> TextRange {
        let offset = self.snap_to_boundary(offset);
        match unit {
            DeletionUnit::Grapheme => match direction {
                EditDirection::Backward => TextRange::new(self.prev_boundary(offset), offset),
                EditDirection::Forward => TextRange::new(offset, self.next_boundary(offset)),
            },
            DeletionUnit::Word => match direction {
                EditDirection::Backward => TextRange::new(self.prev_word(offset), offset),
                EditDirection::Forward => TextRange::new(offset, self.next_word(offset)),
            },
            DeletionUnit::Line => TextRange::new(self.line_start(offset), self.line_end(offset)),
            DeletionUnit::ToLineStart => TextRange::new(self.line_start(offset), offset),
            DeletionUnit::ToLineEnd => TextRange::new(offset, self.line_end(offset)),
            DeletionUnit::ToDocumentStart => TextRange::new(0, offset),
            DeletionUnit::ToDocumentEnd => TextRange::new(offset, self.text.len()),
        }
    }

    /// Returns the character boundary immediately before `offset`, or `0`.
    fn prev_boundary(&self, offset: usize) -> usize {
        if offset == 0 {
            return 0;
        }
        let mut i = offset - 1;
        while i > 0 && !self.text.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    /// Returns the character boundary immediately after `offset`, or `len`.
    fn next_boundary(&self, offset: usize) -> usize {
        let len = self.text.len();
        if offset >= len {
            return len;
        }
        let mut i = offset + 1;
        while i < len && !self.text.is_char_boundary(i) {
            i += 1;
        }
        i
    }

    /// Byte offset of the start of the logical line containing `offset` (the
    /// byte after the previous `'\n'`, or `0`).
    fn line_start(&self, offset: usize) -> usize {
        self.text[..offset].rfind('\n').map_or(0, |i| i + 1)
    }

    /// Byte offset of the end of the logical line containing `offset` (the byte
    /// of the next `'\n'`, or `len`).
    fn line_end(&self, offset: usize) -> usize {
        self.text[offset..]
            .find('\n')
            .map_or(self.text.len(), |i| offset + i)
    }

    /// Simplified forward word motion: skip a run of whitespace then a run of
    /// non-whitespace, returning the resulting offset.
    fn next_word(&self, offset: usize) -> usize {
        let mut consumed = 0;
        // Phase 1: leading whitespace.
        for c in self.text[offset..].chars() {
            if c.is_whitespace() {
                consumed += c.len_utf8();
            } else {
                break;
            }
        }
        // Phase 2: the following run of non-whitespace.
        for c in self.text[offset + consumed..].chars() {
            if c.is_whitespace() {
                break;
            }
            consumed += c.len_utf8();
        }
        offset + consumed
    }

    /// Simplified backward word motion: skip a run of whitespace then a run of
    /// non-whitespace (walking backward), returning the resulting offset.
    fn prev_word(&self, offset: usize) -> usize {
        let mut pos = offset;
        // Phase 1: trailing whitespace.
        for c in self.text[..pos].chars().rev() {
            if c.is_whitespace() {
                pos -= c.len_utf8();
            } else {
                break;
            }
        }
        // Phase 2: the preceding run of non-whitespace.
        for c in self.text[..pos].chars().rev() {
            if c.is_whitespace() {
                break;
            }
            pos -= c.len_utf8();
        }
        pos
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn insert(text: &str) -> EditCommand {
        EditCommand::insert(text)
    }

    fn del(unit: DeletionUnit, direction: EditDirection) -> EditCommand {
        EditCommand::Delete { unit, direction }
    }

    fn mov(unit: NavigationUnit, direction: EditDirection, extend: bool) -> EditCommand {
        EditCommand::Move {
            unit,
            direction,
            extend,
        }
    }

    #[test]
    fn new_and_default_are_empty_with_caret_at_zero() {
        let b = TextBuffer::new();
        assert_eq!(b.text(), "");
        assert_eq!(b.len(), 0);
        assert!(b.is_empty());
        assert_eq!(b.selection(), TextSelection::caret(0));
        assert!(!b.can_undo());
        assert!(!b.can_redo());
        assert_eq!(TextBuffer::default(), TextBuffer::new());
    }

    #[test]
    fn from_text_starts_with_caret_at_zero() {
        let b = TextBuffer::from_text("hello");
        assert_eq!(b.text(), "hello");
        assert_eq!(b.len(), 5);
        assert!(!b.is_empty());
        assert_eq!(b.selection(), TextSelection::caret(0));
    }

    #[test]
    fn insert_at_caret_advances_caret() {
        let mut b = TextBuffer::from_text("ac");
        b.set_selection(TextSelection::caret(1));
        b.apply(insert("b"));
        assert_eq!(b.text(), "abc");
        assert_eq!(b.selection(), TextSelection::caret(2));
        assert!(b.can_undo());
    }

    #[test]
    fn insert_replaces_selection() {
        let mut b = TextBuffer::from_text("hello world");
        b.set_selection(TextSelection::new(0, 5)); // "hello"
        b.apply(insert("HI"));
        assert_eq!(b.text(), "HI world");
        // Caret at end of inserted text.
        assert_eq!(b.selection(), TextSelection::caret(2));
    }

    #[test]
    fn replace_command_does_not_coalesce_and_collapses_to_end() {
        let mut b = TextBuffer::from_text("abcdef");
        b.apply(EditCommand::Replace {
            range: TextRange::new(1, 3),
            text: "XYZ".into(),
        });
        assert_eq!(b.text(), "aXYZdef");
        assert_eq!(b.selection(), TextSelection::caret(4));
        // A second replace stays a separate undo unit (Other never coalesces).
        b.apply(EditCommand::Replace {
            range: TextRange::new(0, 1),
            text: "q".into(),
        });
        assert_eq!(b.text(), "qXYZdef");
        b.undo();
        assert_eq!(b.text(), "aXYZdef");
        b.undo();
        assert_eq!(b.text(), "abcdef");
    }

    #[test]
    fn backspace_removes_one_multibyte_char_cleanly() {
        let mut b = TextBuffer::from_text("café");
        assert_eq!(b.len(), 5); // é is two bytes
        b.set_selection(TextSelection::caret(5));
        b.apply(del(DeletionUnit::Grapheme, EditDirection::Backward));
        assert_eq!(b.text(), "caf");
        assert_eq!(b.len(), 3); // dropped two bytes, not one
        assert_eq!(b.selection(), TextSelection::caret(3));
    }

    #[test]
    fn backspace_at_start_is_noop() {
        let mut b = TextBuffer::from_text("abc");
        b.set_selection(TextSelection::caret(0));
        b.apply(del(DeletionUnit::Grapheme, EditDirection::Backward));
        assert_eq!(b.text(), "abc");
        assert!(!b.can_undo());
    }

    #[test]
    fn delete_forward_removes_next_char() {
        let mut b = TextBuffer::from_text("abc");
        b.set_selection(TextSelection::caret(1));
        b.apply(del(DeletionUnit::Grapheme, EditDirection::Forward));
        assert_eq!(b.text(), "ac");
        assert_eq!(b.selection(), TextSelection::caret(1));
    }

    #[test]
    fn delete_of_selection_ignores_unit() {
        let mut b = TextBuffer::from_text("hello");
        b.set_selection(TextSelection::new(1, 4)); // "ell"
        b.apply(del(DeletionUnit::Grapheme, EditDirection::Backward));
        assert_eq!(b.text(), "ho");
        assert_eq!(b.selection(), TextSelection::caret(1));
    }

    #[test]
    fn word_delete_backward() {
        let mut b = TextBuffer::from_text("hello world");
        b.set_selection(TextSelection::caret(11));
        b.apply(del(DeletionUnit::Word, EditDirection::Backward));
        assert_eq!(b.text(), "hello ");
    }

    #[test]
    fn word_delete_forward() {
        let mut b = TextBuffer::from_text("hello world");
        b.set_selection(TextSelection::caret(0));
        b.apply(del(DeletionUnit::Word, EditDirection::Forward));
        assert_eq!(b.text(), " world");
    }

    #[test]
    fn to_line_start_and_end_on_multiline() {
        let text = "first\nsecond line\nthird";
        // caret inside "second line" — offset of 'c' in second: line is bytes 6..17
        let mut b = TextBuffer::from_text(text);
        b.set_selection(TextSelection::caret(9)); // within "second line"
        b.apply(del(DeletionUnit::ToLineStart, EditDirection::Backward));
        assert_eq!(b.text(), "first\nond line\nthird");

        let mut b2 = TextBuffer::from_text(text);
        b2.set_selection(TextSelection::caret(9));
        b2.apply(del(DeletionUnit::ToLineEnd, EditDirection::Forward));
        assert_eq!(b2.text(), "first\nsec\nthird");
    }

    #[test]
    fn delete_whole_line() {
        let text = "aaa\nbbb\nccc";
        let mut b = TextBuffer::from_text(text);
        b.set_selection(TextSelection::caret(5)); // within "bbb"
        b.apply(del(DeletionUnit::Line, EditDirection::Backward));
        assert_eq!(b.text(), "aaa\n\nccc");
    }

    #[test]
    fn navigation_grapheme_offsets() {
        let b = TextBuffer::from_text("café");
        assert_eq!(
            b.resolve_navigation(3, NavigationUnit::Grapheme, EditDirection::Forward),
            5
        );
        assert_eq!(
            b.resolve_navigation(5, NavigationUnit::Grapheme, EditDirection::Backward),
            3
        );
        // Clamped at the ends.
        assert_eq!(
            b.resolve_navigation(0, NavigationUnit::Grapheme, EditDirection::Backward),
            0
        );
        assert_eq!(
            b.resolve_navigation(5, NavigationUnit::Grapheme, EditDirection::Forward),
            5
        );
    }

    #[test]
    fn navigation_word_offsets() {
        let b = TextBuffer::from_text("hello world");
        assert_eq!(
            b.resolve_navigation(0, NavigationUnit::Word, EditDirection::Forward),
            5
        );
        assert_eq!(
            b.resolve_navigation(5, NavigationUnit::Word, EditDirection::Forward),
            11
        );
        assert_eq!(
            b.resolve_navigation(11, NavigationUnit::Word, EditDirection::Backward),
            6
        );
        assert_eq!(
            b.resolve_navigation(6, NavigationUnit::Word, EditDirection::Backward),
            0
        );
    }

    #[test]
    fn navigation_line_boundary_and_document() {
        let text = "abc\ndefgh\nij";
        let b = TextBuffer::from_text(text);
        // Within "defgh" (bytes 4..9).
        assert_eq!(
            b.resolve_navigation(6, NavigationUnit::LineBoundary, EditDirection::Backward),
            4
        );
        assert_eq!(
            b.resolve_navigation(6, NavigationUnit::LineBoundary, EditDirection::Forward),
            9
        );
        assert_eq!(
            b.resolve_navigation(6, NavigationUnit::Document, EditDirection::Backward),
            0
        );
        assert_eq!(
            b.resolve_navigation(6, NavigationUnit::Document, EditDirection::Forward),
            text.len()
        );
    }

    #[test]
    fn move_without_extend_collapses() {
        let mut b = TextBuffer::from_text("hello");
        b.set_selection(TextSelection::new(0, 5));
        b.apply(mov(
            NavigationUnit::Grapheme,
            EditDirection::Backward,
            false,
        ));
        // Focus was 5, moves to 4, collapses.
        assert_eq!(b.selection(), TextSelection::caret(4));
        // Move does not touch history.
        assert!(!b.can_undo());
    }

    #[test]
    fn move_with_extend_builds_selection() {
        let mut b = TextBuffer::from_text("hello world");
        b.set_selection(TextSelection::caret(0));
        b.apply(mov(NavigationUnit::Word, EditDirection::Forward, true));
        let sel = b.selection();
        assert_eq!(sel.anchor(), 0);
        assert_eq!(sel.focus(), 5);
        assert!(!sel.is_collapsed());
    }

    #[test]
    fn select_all_and_set_selection_route_through_clamp() {
        let mut b = TextBuffer::from_text("café");
        b.apply(EditCommand::SelectAll);
        assert_eq!(b.selection(), TextSelection::new(0, 5));

        // SetSelection clamps out-of-range and snaps off a non-boundary.
        b.apply(EditCommand::SetSelection(TextSelection::new(4, 100)));
        // 4 is inside é (bytes 3..5) -> snaps down to 3; 100 clamps to 5.
        assert_eq!(b.selection(), TextSelection::new(3, 5));
    }

    #[test]
    fn move_does_not_clear_redo() {
        let mut b = TextBuffer::new();
        b.apply(insert("a"));
        b.undo();
        assert!(b.can_redo());
        b.apply(mov(NavigationUnit::Document, EditDirection::Forward, false));
        // Navigation must not discard the redo stack.
        assert!(b.can_redo());
    }

    #[test]
    fn undo_redo_round_trip_restores_text_and_selection() {
        let mut b = TextBuffer::from_text("world");
        b.set_selection(TextSelection::caret(0));
        b.apply(insert("hello "));
        assert_eq!(b.text(), "hello world");
        let after = b.selection();

        assert!(b.undo());
        assert_eq!(b.text(), "world");
        assert_eq!(b.selection(), TextSelection::caret(0)); // selection_before

        assert!(b.redo());
        assert_eq!(b.text(), "hello world");
        assert_eq!(b.selection(), after); // selection_after
    }

    #[test]
    fn typing_coalesces_into_one_undo_unit() {
        let mut b = TextBuffer::new();
        b.apply(insert("a"));
        b.apply(insert("b"));
        b.apply(insert("c"));
        assert_eq!(b.text(), "abc");
        // Three sequential inserts coalesce -> a single undo clears all three.
        assert!(b.undo());
        assert_eq!(b.text(), "");
        assert!(!b.can_undo());
        // And a single redo restores all three.
        assert!(b.redo());
        assert_eq!(b.text(), "abc");
        assert!(!b.can_redo());
    }

    #[test]
    fn typing_run_broken_by_caret_jump_starts_new_unit() {
        let mut b = TextBuffer::from_text("XY");
        b.set_selection(TextSelection::caret(0));
        b.apply(insert("a")); // "aXY", caret 1
        // Jump the caret so the next insert is not contiguous.
        b.set_selection(TextSelection::caret(3));
        b.apply(insert("b")); // "aXYb", caret 4
        assert_eq!(b.text(), "aXYb");
        b.undo();
        assert_eq!(b.text(), "aXY");
        b.undo();
        assert_eq!(b.text(), "XY");
    }

    #[test]
    fn deletion_coalesces_backspace_run() {
        let mut b = TextBuffer::from_text("abc");
        b.set_selection(TextSelection::caret(3));
        b.apply(del(DeletionUnit::Grapheme, EditDirection::Backward));
        b.apply(del(DeletionUnit::Grapheme, EditDirection::Backward));
        b.apply(del(DeletionUnit::Grapheme, EditDirection::Backward));
        assert_eq!(b.text(), "");
        // One undo restores the whole deleted run.
        assert!(b.undo());
        assert_eq!(b.text(), "abc");
        assert!(!b.can_undo());
    }

    #[test]
    fn deletion_coalesces_forward_delete_run() {
        let mut b = TextBuffer::from_text("abc");
        b.set_selection(TextSelection::caret(0));
        b.apply(del(DeletionUnit::Grapheme, EditDirection::Forward));
        b.apply(del(DeletionUnit::Grapheme, EditDirection::Forward));
        assert_eq!(b.text(), "c");
        assert!(b.undo());
        assert_eq!(b.text(), "abc");
        assert!(!b.can_undo());
    }

    #[test]
    fn typing_and_deletion_do_not_coalesce_together() {
        let mut b = TextBuffer::new();
        b.apply(insert("a"));
        b.apply(insert("b")); // one typing unit
        b.apply(del(DeletionUnit::Grapheme, EditDirection::Backward)); // separate deletion unit
        assert_eq!(b.text(), "a");
        b.undo(); // undo the deletion
        assert_eq!(b.text(), "ab");
        b.undo(); // undo the typing run
        assert_eq!(b.text(), "");
    }

    #[test]
    fn new_edit_clears_redo() {
        let mut b = TextBuffer::new();
        b.apply(insert("a"));
        b.undo();
        assert!(b.can_redo());
        b.apply(insert("z"));
        assert!(!b.can_redo());
        assert_eq!(b.text(), "z");
    }

    #[test]
    fn undo_and_redo_on_empty_stack_return_false() {
        let mut b = TextBuffer::new();
        assert!(!b.undo());
        assert!(!b.redo());
    }

    #[test]
    fn replace_that_changes_nothing_is_noop() {
        let mut b = TextBuffer::from_text("abc");
        b.apply(EditCommand::Replace {
            range: TextRange::new(0, 1),
            text: "a".into(),
        });
        // Identical bytes: no history recorded.
        assert!(!b.can_undo());
        assert_eq!(b.text(), "abc");
    }

    #[test]
    fn apply_is_deterministic() {
        let run = || {
            let mut b = TextBuffer::from_text("séed");
            b.set_selection(TextSelection::caret(0));
            b.apply(insert("x"));
            b.apply(mov(NavigationUnit::Word, EditDirection::Forward, true));
            b.apply(del(DeletionUnit::Grapheme, EditDirection::Backward));
            (b.text().to_owned(), b.selection())
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn multibyte_undo_round_trip() {
        let mut b = TextBuffer::from_text("aée");
        b.set_selection(TextSelection::caret(b.len()));
        b.apply(insert("💧")); // 4-byte scalar
        let full = b.text().to_owned();
        assert!(b.undo());
        assert_eq!(b.text(), "aée");
        assert!(b.redo());
        assert_eq!(b.text(), full);
    }
}
