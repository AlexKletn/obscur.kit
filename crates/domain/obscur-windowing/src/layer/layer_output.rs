#![doc = include_str!("layer_output.doc/_module.md")]

#[doc = include_str!("layer_output.doc/LayerOutput.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum LayerOutput {
    #[doc = include_str!("layer_output.doc/Compositor.md")]
    #[default]
    Compositor,
    #[doc = include_str!("layer_output.doc/Named.md")]
    Named(String),
}

impl LayerOutput {
    #[doc = include_str!("layer_output.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Compositor => None,
            Self::Named(name) => Some(name),
        }
    }

    #[doc = include_str!("layer_output.doc/is_compositor_choice.md")]
    #[must_use]
    pub const fn is_compositor_choice(&self) -> bool {
        matches!(self, Self::Compositor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_leaves_the_choice_to_the_compositor() {
        assert!(LayerOutput::default().is_compositor_choice());
        assert_eq!(LayerOutput::default().name(), None);
    }

    #[test]
    fn a_named_output_carries_its_connector() {
        let output = LayerOutput::Named("eDP-1".to_owned());
        assert_eq!(output.name(), Some("eDP-1"));
        assert!(!output.is_compositor_choice());
    }
}
