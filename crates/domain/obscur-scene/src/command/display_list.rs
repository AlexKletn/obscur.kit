//! An ordered, immutable sequence of paint commands.

use core::slice::Iter;

use super::PaintCommand;

#[doc = include_str!("../../docs/api/command/display_list.doc/DisplayList.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DisplayList {
    commands: Vec<PaintCommand>,
}

impl DisplayList {
    /// Creates an empty display list.
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Creates a display list from an existing sequence of commands, preserving
    /// their order as the paint order. The commands are taken verbatim and are
    /// not validated for balance.
    #[must_use]
    pub fn from_commands(commands: Vec<PaintCommand>) -> Self {
        Self { commands }
    }

    #[doc = include_str!("../../docs/api/command/display_list.doc/into_commands.md")]
    #[must_use]
    pub fn into_commands(self) -> Vec<PaintCommand> {
        self.commands
    }

    /// The commands in paint order.
    #[must_use]
    pub fn commands(&self) -> &[PaintCommand] {
        &self.commands
    }

    /// Iterates over the commands in paint order.
    pub fn iter(&self) -> Iter<'_, PaintCommand> {
        self.commands.iter()
    }

    /// The number of commands in the list.
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Whether the list contains no commands.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    #[doc = include_str!("../../docs/api/command/display_list.doc/is_balanced.md")]
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        Self::commands_are_balanced(&self.commands)
    }

    pub(super) fn commands_are_balanced(commands: &[PaintCommand]) -> bool {
        let mut scopes = Vec::new();
        for command in commands {
            if command.is_push() {
                scopes.push(command);
            } else if command.is_pop()
                && !scopes.pop().is_some_and(|open| command.closes_scope(open))
            {
                return false;
            }
        }
        scopes.is_empty()
    }
}

impl<'a> IntoIterator for &'a DisplayList {
    type Item = &'a PaintCommand;
    type IntoIter = Iter<'a, PaintCommand>;

    fn into_iter(self) -> Self::IntoIter {
        self.commands.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clip::Clip;
    use crate::color::Color;
    use crate::layer::SceneLayer;
    use crate::paint::Paint;
    use crate::path::Path;
    use obscur_geometry::Rect;

    fn draw() -> PaintCommand {
        PaintCommand::DrawPath {
            path: Path::rect(Rect::default()),
            paint: Paint::solid_fill(Color::WHITE),
        }
    }

    #[test]
    fn equal_push_and_pop_counts_do_not_make_crossed_scopes_balanced() {
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushLayer(SceneLayer::new()),
            PaintCommand::PushClip(Clip::rect(Rect::default())),
            PaintCommand::PopLayer,
            PaintCommand::PopClip,
        ]);
        assert!(!list.is_balanced());
    }

    #[test]
    fn new_and_default_are_empty() {
        assert!(DisplayList::new().is_empty());
        assert_eq!(DisplayList::new(), DisplayList::default());
        assert_eq!(DisplayList::new().len(), 0);
    }

    #[test]
    fn empty_list_is_balanced() {
        assert!(DisplayList::new().is_balanced());
    }

    #[test]
    fn from_commands_preserves_order_and_reports_balance() {
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushLayer(SceneLayer::new()),
            PaintCommand::PushClip(Clip::rect(Rect::default())),
            draw(),
            PaintCommand::PopClip,
            PaintCommand::PopLayer,
        ]);
        assert_eq!(list.len(), 5);
        assert!(list.commands()[0].is_push());
        assert_eq!(list.iter().count(), 5);
        assert!(list.is_balanced());
    }

    #[test]
    fn into_commands_hands_back_the_buffer_it_was_given() {
        // Why this is worth a test rather than being obvious from the signature:
        // `into_commands` and `commands().to_vec()` produce equal `Vec`s, so no
        // assertion about CONTENT can tell them apart, and the composer reached for
        // the second on an owned list it was about to drop — a deep copy of every
        // command of every painted node, once per node per frame. Pointer identity
        // is the only observable difference, so it is the one asserted.
        let commands = vec![draw(), draw(), draw()];
        let source = commands.as_ptr();
        let list = DisplayList::from_commands(commands);

        let copied = list.commands().to_vec();
        assert!(
            !core::ptr::eq(copied.as_ptr(), source),
            "`commands().to_vec()` is a copy — two live buffers cannot share an address"
        );
        drop(copied);

        let moved = list.into_commands();
        assert!(
            core::ptr::eq(moved.as_ptr(), source),
            "`into_commands` must move the buffer out, not copy it"
        );
    }

    #[test]
    fn missing_pop_is_unbalanced() {
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushClip(Clip::rect(Rect::default())),
            draw(),
        ]);
        assert!(!list.is_balanced());
    }

    #[test]
    fn stray_pop_is_unbalanced() {
        let list = DisplayList::from_commands(vec![PaintCommand::PopClip]);
        assert!(!list.is_balanced());
    }
}
