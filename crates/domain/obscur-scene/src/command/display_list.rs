//! An ordered, immutable sequence of paint commands.

use core::slice::Iter;

use super::PaintCommand;

/// An ordered, immutable sequence of [`PaintCommand`]s with a **deterministic
/// paint order** (spec §5.1).
///
/// The order of the commands *is* the paint order: a renderer executes them
/// front to back, so the same list renders identically across backends. The
/// list is a flat stream in which [`PaintCommand::PushClip`] /
/// [`PaintCommand::PushLayer`] open nested regions closed by their matching
/// pops.
///
/// A **well-formed** list is *balanced* — see [`is_balanced`]. Renderers rely
/// on this invariant; an unbalanced list has a dangling push or a stray pop and
/// cannot be consumed safely. Balance is checkable, not enforced: constructing
/// a `DisplayList` never validates it.
///
/// [`is_balanced`]: DisplayList::is_balanced
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

    /// Whether the list is balanced: every [`PaintCommand::PushClip`] /
    /// [`PaintCommand::PushLayer`] has a matching pop, pushes and pops nest
    /// correctly, and the running nesting depth never drops below zero.
    ///
    /// Computed by folding [`PaintCommand::nesting_delta`] over the stream: the
    /// running depth must never go negative (a stray pop) and must end at zero
    /// (no dangling push). A well-formed list — such as one produced by a
    /// balanced [`DisplayListBuilder`](crate::command::DisplayListBuilder) — is
    /// balanced.
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        let mut depth: i32 = 0;
        for command in &self.commands {
            depth += command.nesting_delta();
            if depth < 0 {
                return false;
            }
        }
        depth == 0
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
