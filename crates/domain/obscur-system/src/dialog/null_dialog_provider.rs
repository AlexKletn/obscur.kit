use super::{DialogProvider, DialogRequest, DialogResult};
use obscur_capability::SupportLevel;

#[doc = include_str!("../../docs/api/dialog/dialog_provider.doc/NullDialogProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct NullDialogProvider;

impl DialogProvider for NullDialogProvider {
    fn origin(&self) -> &'static str {
        "null"
    }

    fn present(&mut self, _request: DialogRequest) -> DialogResult {
        // Honest: a null provider can present no dialog.
        DialogResult::Unsupported
    }

    fn supports(&self, _request: &DialogRequest) -> SupportLevel {
        SupportLevel::Unavailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_provider_is_honestly_unsupported() {
        let mut provider = NullDialogProvider;
        assert_eq!(provider.origin(), "null");
        assert_eq!(
            provider.present(DialogRequest::open_file()),
            DialogResult::Unsupported
        );
        assert_eq!(
            provider.supports(&DialogRequest::open_file()),
            SupportLevel::Unavailable
        );
    }
}
