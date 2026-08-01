#![doc = include_str!("codegen_root.doc/_module.md")]

#[doc = include_str!("codegen_root.doc/CodegenRoot.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenRoot {
    view: Box<str>,
    identity: Box<str>,
}

impl CodegenRoot {
    #[doc = include_str!("codegen_root.doc/facade.md")]
    #[must_use]
    pub fn facade() -> Self {
        Self::new("::obscur_kit::view::view", "::obscur_kit::identity")
    }

    #[doc = include_str!("codegen_root.doc/internal.md")]
    #[must_use]
    pub fn internal() -> Self {
        Self::new("::obscur_view::view", "::obscur_identity")
    }

    #[doc = include_str!("codegen_root.doc/new.md")]
    #[must_use]
    pub fn new(view: impl Into<Box<str>>, identity: impl Into<Box<str>>) -> Self {
        Self {
            view: view.into(),
            identity: identity.into(),
        }
    }

    #[doc = include_str!("codegen_root.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> &str {
        &self.view
    }

    #[doc = include_str!("codegen_root.doc/identity.md")]
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
