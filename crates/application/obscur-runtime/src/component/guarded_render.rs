#![doc = include_str!("guarded_render.doc/_module.md")]

use obscur_view::boundary::ComponentFailure;
use obscur_view::view::View;

use crate::recovery::RecoveryResult;

#[doc = include_str!("guarded_render.doc/GuardedRender.md")]
#[derive(Debug)]
pub enum GuardedRender {
    #[doc = include_str!("guarded_render.doc/Rendered.md")]
    Rendered(View),
    #[doc = include_str!("guarded_render.doc/Failed.md")]
    Failed {
        #[doc = include_str!("guarded_render.doc/failure.md")]
        failure: ComponentFailure,
        #[doc = include_str!("guarded_render.doc/result.md")]
        result: RecoveryResult,
        #[doc = include_str!("guarded_render.doc/fallback.md")]
        fallback: Option<View>,
    },
}

impl GuardedRender {
    #[doc = include_str!("guarded_render.doc/rendered.md")]
    #[must_use]
    pub fn rendered(self) -> Option<View> {
        match self {
            Self::Rendered(view) => Some(view),
            Self::Failed { .. } => None,
        }
    }
}
