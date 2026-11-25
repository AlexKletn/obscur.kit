use std::sync::Arc;

#[doc = include_str!("../../docs/api/appearance/appearance_provider.doc/AppearanceCallback.md")]
pub type AppearanceCallback = Arc<dyn Fn() + Send + Sync + 'static>;
