#![doc = include_str!("mod.doc/_module.md")]
mod diagnostic_filter;

pub use diagnostic_filter::DiagnosticFilter;

mod diagnostic_buffer_policy;
mod diagnostic_channel;
mod diagnostic_subscription;

pub use diagnostic_buffer_policy::{DiagnosticBufferPolicy, OverflowStrategy};
pub use diagnostic_channel::{ChannelSink, DiagnosticChannel};
pub use diagnostic_subscription::{DiagnosticSubscription, SubscriptionId};
