#![doc = include_str!("style_delta.doc/_module.md")]

use crate::orchestration::ComputedStyles;

#[doc = include_str!("style_delta.doc/StyleDelta.md")]
pub(crate) enum StyleDelta {
    #[doc = include_str!("style_delta.doc/Full.md")]
    Full,
    #[doc = include_str!("style_delta.doc/Incremental.md")]
    Incremental {
        #[doc = include_str!("style_delta.doc/replaced.md")]
        replaced: ComputedStyles,
        #[doc = include_str!("style_delta.doc/introduced.md")]
        introduced: bool,
    },
}
