use obscur_plugin_api::status::PluginApiError;

#[derive(Default)]
pub(super) enum Activation {
    #[default]
    Idle,
    NeedsCleanup,
    Ready,
}

impl Activation {
    pub(super) fn initialize(
        &mut self,
        callback: impl FnOnce() -> Result<(), PluginApiError>,
    ) -> Result<(), PluginApiError> {
        if !matches!(self, Self::Idle) {
            return Err(PluginApiError::AlreadyActive);
        }
        // Guest callbacks can acquire resources before reporting an error.
        *self = Self::NeedsCleanup;
        callback()?;
        *self = Self::Ready;
        Ok(())
    }

    pub(super) const fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }

    pub(super) fn take_cleanup(&mut self) -> bool {
        !matches!(std::mem::take(self), Self::Idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_initialization_keeps_the_error_and_requires_cleanup() {
        let mut activation = Activation::default();
        let failure = PluginApiError::Rejected("initialization failed".into());
        assert_eq!(activation.initialize(|| Err(failure.clone())), Err(failure));
        assert!(!activation.is_ready());
        assert_eq!(
            activation.initialize(|| panic!("cleanup must run first")),
            Err(PluginApiError::AlreadyActive)
        );
        assert!(activation.take_cleanup());
        assert!(!activation.take_cleanup());
        activation.initialize(|| Ok(())).unwrap();
        assert!(activation.is_ready());
    }
}
