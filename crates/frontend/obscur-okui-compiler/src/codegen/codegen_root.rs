//! [`CodegenRoot`]: the crate paths generated code names the engine's types
//! through.

/// The two absolute paths [`ViewCodegen`](super::ViewCodegen) writes its emitted
/// types with: the module holding `obscur-view`'s view types, and the crate
/// holding `obscur-identity`'s handles.
///
/// # Why this is an argument and not a constant
///
/// It used to be a constant — the emitted paths were the literals
/// `::obscur_view::view::…` and `::obscur_identity::…` — and that made the
/// primary authoring surface unusable outside this workspace. An application
/// depends on the `obscur-kit` facade and nothing else, exactly as
/// `docs/api/README.md` tells it to; the facade re-exports those crates under
/// *different* names (`obscur_kit::view`, `obscur_kit::identity`), so the first
/// `okui!` in an application failed with
///
/// ```text
/// error[E0433]: cannot find `obscur_view` in the crate root
/// ```
///
/// A `::`-prefixed path resolves in the **extern prelude**, which holds a
/// crate's *direct* dependencies only, so re-exporting under the expected names
/// could not have fixed it: no re-export puts `obscur_view` there. The emitted
/// root has to be the caller's, so the caller states it.
///
/// That the defect survived to be found from outside is structural: every
/// in-repo consumer of the codegen lives *inside* `obscur-kit`, where both
/// crates are direct dependencies, so the failing path was never taken. The
/// argument is modelled on
/// [`TagPosition`](obscur_ui_ir::name_binding::TagPosition) for the same reason
/// it exists there — a caller must *state* which resolution it wants, so the
/// wrong one can no longer be picked by omission.
///
/// # Which one to pass
///
/// [`facade`](Self::facade) for anything an application compiles: its own
/// `okui!` call sites and its `build.rs`-generated files alike, since
/// `obscur-kit` is the one crate it depends on. [`internal`](Self::internal)
/// only for a crate that depends on `obscur-view` and `obscur-identity`
/// directly. [`new`](Self::new) for a graph that re-exports them somewhere else
/// again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenRoot {
    view: Box<str>,
    identity: Box<str>,
}

impl CodegenRoot {
    /// Paths through the `obscur-kit` facade: `::obscur_kit::view::view` and
    /// `::obscur_kit::identity`.
    ///
    /// The right choice for an application, which depends on the facade alone.
    #[must_use]
    pub fn facade() -> Self {
        Self::new("::obscur_kit::view::view", "::obscur_kit::identity")
    }

    /// Paths straight to the internal crates: `::obscur_view::view` and
    /// `::obscur_identity`.
    ///
    /// Only correct for a crate that has both as *direct* dependencies — which
    /// an application does not, and should not need to.
    #[must_use]
    pub fn internal() -> Self {
        Self::new("::obscur_view::view", "::obscur_identity")
    }

    /// Custom paths: `view` is the module holding `View`/`ViewNode`/`ViewValue`,
    /// `identity` the one holding `ComponentId`/`InternedSymbol`.
    ///
    /// Both are written into generated code verbatim, so pass absolute paths
    /// (leading `::`) unless the generated file is `include!`d somewhere a
    /// relative one resolves.
    #[must_use]
    pub fn new(view: impl Into<Box<str>>, identity: impl Into<Box<str>>) -> Self {
        Self {
            view: view.into(),
            identity: identity.into(),
        }
    }

    /// The module path the view types are named through.
    #[must_use]
    pub fn view(&self) -> &str {
        &self.view
    }

    /// The crate path the identity handles are named through.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_facade_root_names_the_umbrella_crate() {
        let root = CodegenRoot::facade();
        assert_eq!(root.view(), "::obscur_kit::view::view");
        assert_eq!(root.identity(), "::obscur_kit::identity");
    }

    #[test]
    fn the_internal_root_names_the_crates_directly() {
        let root = CodegenRoot::internal();
        assert_eq!(root.view(), "::obscur_view::view");
        assert_eq!(root.identity(), "::obscur_identity");
    }

    #[test]
    fn the_two_shipped_roots_differ() {
        // The whole point: one of them resolves in an application's crate root
        // and the other does not, so they cannot be the same strings.
        assert_ne!(CodegenRoot::facade(), CodegenRoot::internal());
    }

    #[test]
    fn a_custom_root_is_kept_verbatim() {
        let root = CodegenRoot::new("::my_kit::v", "::my_kit::ids");
        assert_eq!(root.view(), "::my_kit::v");
        assert_eq!(root.identity(), "::my_kit::ids");
    }
}
