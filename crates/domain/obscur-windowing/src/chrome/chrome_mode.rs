#![doc = include_str!("chrome_mode.doc/_module.md")]

#[doc = include_str!("chrome_mode.doc/ChromeMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChromeMode {
    #[doc = include_str!("chrome_mode.doc/SystemFrame.md")]
    SystemFrame,
    #[doc = include_str!("chrome_mode.doc/ClientSide.md")]
    ClientSide,
    #[doc = include_str!("chrome_mode.doc/Custom.md")]
    Custom,
}

impl ChromeMode {
    #[doc = include_str!("chrome_mode.doc/is_custom.md")]
    #[must_use]
    pub const fn is_custom(self) -> bool {
        matches!(self, Self::Custom)
    }

    #[doc = include_str!("chrome_mode.doc/uses_client_side.md")]
    #[must_use]
    pub const fn uses_client_side(self) -> bool {
        matches!(self, Self::ClientSide | Self::Custom)
    }
}

impl Default for ChromeMode {
    #[doc = include_str!("chrome_mode.doc/default.md")]
    fn default() -> Self {
        Self::SystemFrame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_system_frame() {
        assert_eq!(ChromeMode::default(), ChromeMode::SystemFrame);
    }

    #[test]
    fn predicates() {
        assert!(ChromeMode::Custom.is_custom());
        assert!(!ChromeMode::ClientSide.is_custom());
        assert!(!ChromeMode::SystemFrame.is_custom());

        assert!(ChromeMode::ClientSide.uses_client_side());
        assert!(ChromeMode::Custom.uses_client_side());
        assert!(!ChromeMode::SystemFrame.uses_client_side());
    }
}
