#![doc = include_str!("flex_main_margins.doc/_module.md")]

use crate::model::Margin;

#[doc = include_str!("flex_main_margins.doc/reserved.md")]
pub(super) fn reserved(margin: Margin) -> f32 {
    margin.resolve(0.0)
}

#[doc = include_str!("flex_main_margins.doc/used.md")]
pub(super) fn used(margin: Margin, auto_share: f32) -> f32 {
    if margin.is_auto() {
        auto_share
    } else {
        reserved(margin)
    }
}
