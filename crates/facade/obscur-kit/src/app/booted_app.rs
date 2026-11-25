use super::app_parts::ScrollHandler;
use super::appearance_subscription::AppearanceSubscription;
use crate::bootstrap::{CssPipeline, KitBootstrap};
use crate::control::AppControl;
use crate::runtime::UiRuntime;

#[doc = include_str!("../../docs/api/app/app_parts.doc/BootedApp.md")]
pub(super) struct BootedApp {
    pub(super) runtime: UiRuntime,
    pub(super) appearance: Option<AppearanceSubscription>,
    pub(super) css: CssPipeline,
    #[doc = include_str!("../../docs/api/app/app_parts.doc/boot.md")]
    pub(super) boot: KitBootstrap,
    pub(super) control: AppControl,
    pub(super) scroll: Option<ScrollHandler>,
}
