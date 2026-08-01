#![doc = include_str!("background_size.doc/_module.md")]

use crate::property::Length;

#[doc = include_str!("background_size.doc/BackgroundSize.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BackgroundSize {
    #[doc = include_str!("background_size.doc/Auto.md")]
    #[default]
    Auto,
    #[doc = include_str!("background_size.doc/Cover.md")]
    Cover,
    #[doc = include_str!("background_size.doc/Contain.md")]
    Contain,
    #[doc = include_str!("background_size.doc/Explicit.md")]
    Explicit {
        #[doc = include_str!("background_size.doc/width.md")]
        width: Length,
        #[doc = include_str!("background_size.doc/height.md")]
        height: Length,
    },
}

impl BackgroundSize {
    #[doc = include_str!("background_size.doc/uses_ch.md")]
    #[must_use]
    pub const fn uses_ch(&self) -> bool {
        match self {
            Self::Explicit { width, height } => width.is_ch() || height.is_ch(),
            Self::Auto | Self::Cover | Self::Contain => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_auto() {
        assert_eq!(BackgroundSize::default(), BackgroundSize::Auto);
    }

    #[test]
    fn explicit_carries_both_axes() {
        let size = BackgroundSize::Explicit {
            width: Length::Percent(50.0),
            height: Length::Auto,
        };
        assert_eq!(
            size,
            BackgroundSize::Explicit {
                width: Length::Percent(50.0),
                height: Length::Auto,
            }
        );
        assert_ne!(size, BackgroundSize::Cover);
    }
}
