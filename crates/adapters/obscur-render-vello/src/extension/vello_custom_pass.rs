//! The adapter-side descriptor of a trusted custom render pass (spec §5.3).

/// A named, trusted custom render pass gated by a capability check (spec §5.3).
///
/// Spec §5.3 permits trusted extensions to inject custom render passes, but only
/// behind capability checks — in this backend, the
/// [`custom_shaders`](obscur_render::renderer::RendererCapabilities::has_custom_shaders)
/// capability plus a live GPU device. A real pass records wgpu commands against
/// the device; since the device is host-provided and optional (spec §2.7), this
/// type is the descriptor half: it names the pass and records whether it needs a
/// device, so the runtime can decide up front whether the pass is permitted.
///
/// Wiring an actual `wgpu::RenderPass` is a future refinement; the name and
/// permission contract here is the stable surface a trusted extension registers
/// against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VelloCustomPass {
    name: Box<str>,
    requires_device: bool,
}

impl VelloCustomPass {
    /// Creates a device-requiring custom pass with the given `name`.
    ///
    /// A custom pass records GPU commands, so it defaults to requiring a device;
    /// use [`Self::device_free`] for a pass that does not.
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

    /// Whether the pass is permitted to run given the availability of a device.
    ///
    /// A device-requiring pass (spec §5.3) is permitted only when a device is
    /// present; a device-free pass is always permitted.
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
        assert_ne!(
            VelloCustomPass::new("a"),
            VelloCustomPass::device_free("a")
        );
    }
}
