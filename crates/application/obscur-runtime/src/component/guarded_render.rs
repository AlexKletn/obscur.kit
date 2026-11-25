//! `GuardedRender`: what rendering a component behind its §1.7 boundary produced.

use obscur_view::boundary::ComponentFailure;
use obscur_view::view::View;

use crate::recovery::RecoveryResult;

#[doc = include_str!("../../docs/api/component/guarded_render.doc/GuardedRender.md")]
#[derive(Debug)]
pub enum GuardedRender {
    /// The component rendered: this view is its new subtree.
    Rendered(View),
    /// The render failed and the boundary answered for it.
    Failed {
        #[doc = include_str!("../../docs/api/component/guarded_render.doc/failure.md")]
        failure: ComponentFailure,
        #[doc = include_str!("../../docs/api/component/guarded_render.doc/result.md")]
        result: RecoveryResult,
        /// The fallback UI to commit in place of the failed subtree — `Some`
        /// exactly when `result` is [`RecoveryResult::FellBack`]. A retry leaves
        /// the subtree to the next frame, and a propagated failure leaves it to
        /// an enclosing boundary.
        fallback: Option<View>,
    },
}

impl GuardedRender {
    /// The rendered view, or `None` when the render failed.
    #[must_use]
    pub fn rendered(self) -> Option<View> {
        match self {
            Self::Rendered(view) => Some(view),
            Self::Failed { .. } => None,
        }
    }
}
