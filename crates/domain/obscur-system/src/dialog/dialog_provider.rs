#![doc = include_str!("../../docs/api/dialog/dialog_provider.doc/_module.md")]

use super::{DialogRequest, DialogResult};
use obscur_capability::SupportLevel;

#[doc = include_str!("../../docs/api/dialog/dialog_provider.doc/DialogProvider.md")]
pub trait DialogProvider {
    #[doc = include_str!("../../docs/api/dialog/dialog_provider.doc/origin.md")]
    fn origin(&self) -> &str;

    #[doc = include_str!("../../docs/api/dialog/dialog_provider.doc/present.md")]
    fn present(&mut self, request: DialogRequest) -> DialogResult;

    /// Reports the honest [`SupportLevel`] at which this provider can satisfy
    /// `request` (spec §6.9).
    #[must_use]
    fn supports(&self, request: &DialogRequest) -> SupportLevel;
}

#[cfg(test)]
mod tests {
    use super::super::NullDialogProvider;
    use super::*;

    #[test]
    fn dialog_provider_is_object_safe() {
        let _: Box<dyn DialogProvider> = Box::new(NullDialogProvider);
    }
}
