//! The adapter-side descriptor of a trusted custom render pass (spec §5.3).

#[doc = include_str!("../../docs/api/extension/vello_custom_pass.doc/VelloCustomPass.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VelloCustomPass {
    name: Box<str>,
    requires_device: bool,
}

impl VelloCustomPass {
    #[doc = include_str!("../../docs/api/extension/vello_custom_pass.doc/new.md")]
    #[must_use]
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            requires_device: true,
        }
    }

    /// Creates a custom pass that does not require a GPU device.
    #[must_use]
    pub fn device_free(name: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            requires_device: false,
        }
    }

    /// The pass's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether this pass needs a GPU device to run.
    #[must_use]
    pub const fn requires_device(&self) -> bool {
        self.requires_device
    }

    #[doc = include_str!("../../docs/api/extension/vello_custom_pass.doc/is_permitted.md")]
    #[must_use]
    pub const fn is_permitted(&self, has_device: bool) -> bool {
        !self.requires_device || has_device
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_requires_a_device() {
        let pass = VelloCustomPass::new("bloom");
        assert_eq!(pass.name(), "bloom");
        assert!(pass.requires_device());
    }

    #[test]
    fn device_requiring_pass_permitted_only_with_device() {
        let pass = VelloCustomPass::new("bloom");
        assert!(!pass.is_permitted(false));
        assert!(pass.is_permitted(true));
    }

    #[test]
    fn device_free_pass_always_permitted() {
        let pass = VelloCustomPass::device_free("tint");
        assert!(!pass.requires_device());
        assert!(pass.is_permitted(false));
        assert!(pass.is_permitted(true));
    }

    #[test]
    fn equality_considers_name_and_device_requirement() {
        assert_eq!(VelloCustomPass::new("a"), VelloCustomPass::new("a"));
        assert_ne!(VelloCustomPass::new("a"), VelloCustomPass::new("b"));
        assert_ne!(VelloCustomPass::new("a"), VelloCustomPass::device_free("a"));
    }
}
