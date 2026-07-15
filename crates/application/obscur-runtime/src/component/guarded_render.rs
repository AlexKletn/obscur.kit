//! `GuardedRender`: what rendering a component behind its §1.7 boundary produced.

use obscur_view::boundary::ComponentFailure;
use obscur_view::view::View;

use crate::recovery::RecoveryResult;

/// The outcome of rendering a
/// [`MountedComponent`](crate::component::MountedComponent) with its error
/// boundary applied (spec §1.7).
///
/// A component's render either produces a view or fails, and a failure is never
/// just an absence: the boundary has already decided what happens to the
/// subtree, and the caller is handed both that decision and the structured
/// failure to report.
#[derive(Debug)]
pub enum GuardedRender {
    /// The component rendered: this view is its new subtree.
    Rendered(View),
    /// The render failed and the boundary answered for it.
    Failed {
        /// The structured failure, carrying the [`Diagnostic`] to report
        /// (spec §1.6).
        ///
        /// [`Diagnostic`]: obscur_diagnostics::Diagnostic
        failure: ComponentFailure,
        /// What the boundary decided (spec §1.7: *contain → (retry | fallback)
        /// | propagate*).
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
