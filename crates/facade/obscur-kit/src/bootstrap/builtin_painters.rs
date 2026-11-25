#![doc = include_str!("../../docs/api/bootstrap/builtin_painters.doc/_module.md")]

use crate::runtime::UiRuntime;
use crate::scene::color::Color;
use crate::scene::paint::Brush;
use crate::widgets::progress::{Progress, paint_progress};
use crate::widgets::slider::{Slider, paint_slider};

#[doc = include_str!("../../docs/api/bootstrap/builtin_painters.doc/BuiltinPainters.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuiltinPainters {
    /// The brush colour the fill of a progress bar and the thumb of a slider are
    /// drawn in.
    accent: Color,
}

impl BuiltinPainters {
    #[doc = include_str!("../../docs/api/bootstrap/builtin_painters.doc/ACCENT.md")]
    pub const ACCENT: Color = Color::rgba(
        0x2b as f32 / 255.0,
        0x2a as f32 / 255.0,
        0x27 as f32 / 255.0,
        1.0,
    );

    #[doc = include_str!("../../docs/api/bootstrap/builtin_painters.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            accent: Self::ACCENT,
        }
    }

    /// The same set, drawn in `accent` instead of the default.
    #[must_use]
    pub const fn with_accent(mut self, accent: Color) -> Self {
        self.accent = accent;
        self
    }

    /// The accent this set will register its painters with.
    #[must_use]
    pub const fn accent(&self) -> Color {
        self.accent
    }

    #[doc = include_str!("../../docs/api/bootstrap/builtin_painters.doc/install.md")]
    pub fn install(&self, runtime: &mut UiRuntime) {
        // Registered under the SYMBOL the widget binds on its own canvas, not
        // under a name: `Progress::contents` writes `canvas_painter_prop`, and a
        // node's own declaration is the key the runtime looks up first — a
        // stylesheet cannot redirect the routine that draws a progress bar.
        let accent = self.accent;
        runtime.register_canvas(
            Progress::PAINTER,
            Box::new(move |region, paint| paint_progress(region, paint, Brush::solid(accent))),
        );
        runtime.register_canvas(
            Slider::PAINTER,
            Box::new(move |region, paint| paint_slider(region, paint, Brush::solid(accent))),
        );
    }
}

impl Default for BuiltinPainters {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::BuiltinPainters;
    use crate::geometry::Size;
    use crate::identity::RuntimeId;
    use crate::runtime::UiRuntime;
    use crate::scene::color::Color;

    #[test]
    fn the_default_accent_is_the_ua_sheets_own_ink() {
        // The track the painters draw under the fill is `#2b2a27` at 25% alpha,
        // and the UA sheet's ink is `#2b2a27`. A default accent picked from
        // anywhere else would put a colour on screen that no shipped stylesheet
        // names, out of the box, with nothing to change it to.
        assert_eq!(BuiltinPainters::new().accent(), BuiltinPainters::ACCENT);
        assert_eq!(
            BuiltinPainters::ACCENT,
            Color::rgba(
                0x2b as f32 / 255.0,
                0x2a as f32 / 255.0,
                0x27 as f32 / 255.0,
                1.0
            )
        );
    }

    #[test]
    fn a_host_can_replace_the_accent_without_replacing_the_painters() {
        let red = Color::rgba(1.0, 0.0, 0.0, 1.0);
        assert_eq!(BuiltinPainters::new().with_accent(red).accent(), red);
    }

    #[test]
    fn installing_twice_is_a_replacement_rather_than_a_failure() {
        // Painters are keyed, not claimed: unlike a component id, the second
        // registration under a key wins. That is what lets a host install the
        // built-ins and then override one, which `App::painter_id` relies on.
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(10.0, 10.0));
        BuiltinPainters::new().install(&mut runtime);
        BuiltinPainters::new()
            .with_accent(Color::rgba(1.0, 0.0, 0.0, 1.0))
            .install(&mut runtime);
    }
}
