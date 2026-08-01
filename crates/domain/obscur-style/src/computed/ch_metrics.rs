#![doc = include_str!("ch_metrics.doc/_module.md")]

use crate::computed::ComputedStyle;

#[doc = include_str!("ch_metrics.doc/ChMetrics.md")]
pub trait ChMetrics {
    #[doc = include_str!("ch_metrics.doc/zero_advance.md")]
    fn zero_advance(&self, font: &ComputedStyle) -> Option<f32>;
}
