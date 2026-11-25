//! Per-topic subscription registry and event fan-out (spec §7.6/§7.7).

mod subscription_manager;
pub use subscription_manager::SubscriptionManager;
