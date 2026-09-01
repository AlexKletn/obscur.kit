#![doc = include_str!("system_palette.doc/_module.md")]

use obscur_scene::color::Color;

#[doc = include_str!("system_palette.doc/SystemPalette.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SystemPalette {
    #[doc = include_str!("system_palette.doc/window_background.md")]
    pub window_background: Option<Color>,
    #[doc = include_str!("system_palette.doc/window_foreground.md")]
    pub window_foreground: Option<Color>,
    #[doc = include_str!("system_palette.doc/view_background.md")]
    pub view_background: Option<Color>,
    #[doc = include_str!("system_palette.doc/view_foreground.md")]
    pub view_foreground: Option<Color>,
    #[doc = include_str!("system_palette.doc/selection_background.md")]
    pub selection_background: Option<Color>,
    #[doc = include_str!("system_palette.doc/selection_foreground.md")]
    pub selection_foreground: Option<Color>,
}

impl SystemPalette {
    #[doc = include_str!("system_palette.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.window_background.is_none()
            && self.window_foreground.is_none()
            && self.view_background.is_none()
            && self.view_foreground.is_none()
            && self.selection_background.is_none()
            && self.selection_foreground.is_none()
    }

    #[doc = include_str!("system_palette.doc/stated.md")]
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
