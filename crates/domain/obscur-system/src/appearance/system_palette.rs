//! The colours a desktop states, where it states any.

use obscur_scene::color::Color;

#[doc = include_str!("../../docs/api/appearance/system_palette.doc/SystemPalette.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SystemPalette {
    /// Behind window chrome — toolbars, panels, dialog bodies.
    pub window_background: Option<Color>,
    /// Text and icons on the window background.
    pub window_foreground: Option<Color>,
    #[doc = include_str!("../../docs/api/appearance/system_palette.doc/view_background.md")]
    pub view_background: Option<Color>,
    /// Text inside a view.
    pub view_foreground: Option<Color>,
    /// Behind selected text or a selected row.
    pub selection_background: Option<Color>,
    /// Selected text itself.
    pub selection_foreground: Option<Color>,
}

impl SystemPalette {
    #[doc = include_str!("../../docs/api/appearance/system_palette.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.window_background.is_none()
            && self.window_foreground.is_none()
            && self.view_background.is_none()
            && self.view_foreground.is_none()
            && self.selection_background.is_none()
            && self.selection_foreground.is_none()
    }

    /// How many of the six the desktop actually named.
    #[must_use]
    pub const fn stated(&self) -> usize {
        let mut count = 0;
        if self.window_background.is_some() {
            count += 1;
        }
        if self.window_foreground.is_some() {
            count += 1;
        }
        if self.view_background.is_some() {
            count += 1;
        }
        if self.view_foreground.is_some() {
            count += 1;
        }
        if self.selection_background.is_some() {
            count += 1;
        }
        if self.selection_foreground.is_some() {
            count += 1;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_desktop_that_says_nothing_leaves_an_empty_palette() {
        // The GNOME case, and the reason every field is an `Option`: `gsettings`
        // names the theme and never its colours, so a palette from that source
        // is empty rather than guessed.
        let palette = SystemPalette::default();
        assert!(palette.is_empty());
        assert_eq!(palette.stated(), 0);
    }

    #[test]
    fn a_partial_palette_is_neither_empty_nor_complete() {
        let palette = SystemPalette {
            window_background: Some(Color::rgb(1.0, 1.0, 1.0)),
            ..SystemPalette::default()
        };
        assert!(!palette.is_empty());
        assert_eq!(palette.stated(), 1);
    }

    #[test]
    fn every_field_counts_once() {
        let full = SystemPalette {
            window_background: Some(Color::rgb(0.0, 0.0, 0.0)),
            window_foreground: Some(Color::rgb(1.0, 1.0, 1.0)),
            view_background: Some(Color::rgb(0.1, 0.1, 0.1)),
            view_foreground: Some(Color::rgb(0.9, 0.9, 0.9)),
            selection_background: Some(Color::rgb(0.2, 0.4, 0.8)),
            selection_foreground: Some(Color::rgb(1.0, 1.0, 1.0)),
        };
        assert_eq!(full.stated(), 6);
        assert!(!full.is_empty());
    }
}
