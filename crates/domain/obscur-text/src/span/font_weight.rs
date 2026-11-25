#[doc = include_str!("../../docs/api/span/text_style.doc/FontWeight.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontWeight(u16);

impl FontWeight {
    /// Thin weight (`100`).
    pub const THIN: FontWeight = FontWeight(100);
    /// Extra-light weight (`200`).
    pub const EXTRA_LIGHT: FontWeight = FontWeight(200);
    /// Light weight (`300`).
    pub const LIGHT: FontWeight = FontWeight(300);
    /// Normal (regular) weight (`400`); the default.
    pub const NORMAL: FontWeight = FontWeight(400);
    /// Medium weight (`500`).
    pub const MEDIUM: FontWeight = FontWeight(500);
    /// Semi-bold weight (`600`).
    pub const SEMI_BOLD: FontWeight = FontWeight(600);
    /// Bold weight (`700`).
    pub const BOLD: FontWeight = FontWeight(700);
    /// Extra-bold weight (`800`).
    pub const EXTRA_BOLD: FontWeight = FontWeight(800);
    /// Black (heavy) weight (`900`).
    pub const BLACK: FontWeight = FontWeight(900);

    #[doc = include_str!("../../docs/api/span/text_style.doc/new.md")]
    #[must_use]
    pub const fn new(value: u16) -> Self {
        let clamped = if value < 1 {
            1
        } else if value > 1000 {
            1000
        } else {
            value
        };
        FontWeight(clamped)
    }

    /// Returns the raw weight value on the `1..=1000` axis.
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl Default for FontWeight {
    /// The default weight is [`FontWeight::NORMAL`].
    fn default() -> Self {
        Self::NORMAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_weight_consts() {
        assert_eq!(FontWeight::THIN.value(), 100);
        assert_eq!(FontWeight::NORMAL.value(), 400);
        assert_eq!(FontWeight::BOLD.value(), 700);
        assert_eq!(FontWeight::BLACK.value(), 900);
        assert_eq!(FontWeight::default(), FontWeight::NORMAL);
    }

    #[test]
    fn font_weight_new_clamps() {
        assert_eq!(FontWeight::new(0).value(), 1);
        assert_eq!(FontWeight::new(1).value(), 1);
        assert_eq!(FontWeight::new(450).value(), 450);
        assert_eq!(FontWeight::new(1000).value(), 1000);
        assert_eq!(FontWeight::new(5000).value(), 1000);
    }
}
