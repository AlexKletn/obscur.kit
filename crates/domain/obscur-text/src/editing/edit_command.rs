#![doc = include_str!("edit_command.doc/_module.md")]

use crate::selection::TextSelection;
use crate::span::TextRange;

use super::{DeletionUnit, NavigationUnit};

#[doc = include_str!("edit_command.doc/EditDirection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditDirection {
    #[doc = include_str!("edit_command.doc/Backward.md")]
    Backward,
    #[doc = include_str!("edit_command.doc/Forward.md")]
    Forward,
}

#[doc = include_str!("edit_command.doc/EditCommand.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum EditCommand {
    #[doc = include_str!("edit_command.doc/Insert.md")]
    Insert(Box<str>),
    #[doc = include_str!("edit_command.doc/Delete.md")]
    Delete {
        #[doc = include_str!("edit_command.doc/unit.md")]
        unit: DeletionUnit,
        #[doc = include_str!("edit_command.doc/direction.md")]
        direction: EditDirection,
    },
    #[doc = include_str!("edit_command.doc/Move.md")]
    Move {
        #[doc = include_str!("edit_command.doc/unit_2.md")]
        unit: NavigationUnit,
        #[doc = include_str!("edit_command.doc/direction_2.md")]
        direction: EditDirection,
        #[doc = include_str!("edit_command.doc/extend.md")]
        extend: bool,
    },
    #[doc = include_str!("edit_command.doc/SelectAll.md")]
    SelectAll,
    #[doc = include_str!("edit_command.doc/SetSelection.md")]
    SetSelection(TextSelection),
    #[doc = include_str!("edit_command.doc/Replace.md")]
    Replace {
        #[doc = include_str!("edit_command.doc/range.md")]
        range: TextRange,
        #[doc = include_str!("edit_command.doc/text.md")]
        text: Box<str>,
    },
}

impl EditCommand {
    #[doc = include_str!("edit_command.doc/insert.md")]
    #[must_use]
    pub fn insert(text: impl Into<Box<str>>) -> Self {
        Self::Insert(text.into())
    }

    #[doc = include_str!("edit_command.doc/is_mutating.md")]
    #[must_use]
    pub const fn is_mutating(&self) -> bool {
        matches!(
            self,
            Self::Insert(_) | Self::Delete { .. } | Self::Replace { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_helper_accepts_str_and_string() {
        assert_eq!(EditCommand::insert("hi"), EditCommand::Insert("hi".into()));
        assert_eq!(
            EditCommand::insert(String::from("hi")),
            EditCommand::Insert("hi".into())
        );
    }

    #[test]
    fn mutating_commands() {
        assert!(EditCommand::insert("x").is_mutating());
        assert!(
            EditCommand::Delete {
                unit: DeletionUnit::Grapheme,
                direction: EditDirection::Backward,
            }
            .is_mutating()
        );
        assert!(
            EditCommand::Replace {
                range: TextRange::new(0, 2),
                text: "z".into(),
            }
            .is_mutating()
        );
    }

    #[test]
    fn non_mutating_commands() {
        assert!(
            !EditCommand::Move {
                unit: NavigationUnit::Word,
                direction: EditDirection::Forward,
                extend: true,
            }
            .is_mutating()
        );
        assert!(!EditCommand::SelectAll.is_mutating());
        assert!(!EditCommand::SetSelection(TextSelection::caret(3)).is_mutating());
    }
}
