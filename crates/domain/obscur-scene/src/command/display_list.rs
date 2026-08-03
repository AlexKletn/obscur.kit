#![doc = include_str!("display_list.doc/_module.md")]

use core::slice::Iter;

use super::PaintCommand;

#[doc = include_str!("display_list.doc/DisplayList.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DisplayList {
    commands: Vec<PaintCommand>,
}

impl DisplayList {
    #[doc = include_str!("display_list.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    #[doc = include_str!("display_list.doc/from_commands.md")]
    #[must_use]
    pub fn from_commands(commands: Vec<PaintCommand>) -> Self {
        Self { commands }
    }

    #[doc = include_str!("display_list.doc/into_commands.md")]
    #[must_use]
    pub fn into_commands(self) -> Vec<PaintCommand> {
        self.commands
    }

    #[doc = include_str!("display_list.doc/commands.md")]
    #[must_use]
    pub fn commands(&self) -> &[PaintCommand] {
        &self.commands
    }

    #[doc = include_str!("display_list.doc/iter.md")]
    pub fn iter(&self) -> Iter<'_, PaintCommand> {
        self.commands.iter()
    }

    #[doc = include_str!("display_list.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    #[doc = include_str!("display_list.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    #[doc = include_str!("display_list.doc/is_balanced.md")]
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
