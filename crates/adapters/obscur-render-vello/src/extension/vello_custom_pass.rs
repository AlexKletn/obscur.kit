#![doc = include_str!("vello_custom_pass.doc/_module.md")]

#[doc = include_str!("vello_custom_pass.doc/VelloCustomPass.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VelloCustomPass {
    name: Box<str>,
    requires_device: bool,
}

impl VelloCustomPass {
    #[doc = include_str!("vello_custom_pass.doc/new.md")]
    #[must_use]
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            requires_device: true,
        }
    }

    #[doc = include_str!("vello_custom_pass.doc/device_free.md")]
    #[must_use]
    pub fn device_free(name: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            requires_device: false,
        }
    }

    #[doc = include_str!("vello_custom_pass.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("vello_custom_pass.doc/requires_device.md")]
    #[must_use]
    pub const fn requires_device(&self) -> bool {
        self.requires_device
    }

    #[doc = include_str!("vello_custom_pass.doc/is_permitted.md")]
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
