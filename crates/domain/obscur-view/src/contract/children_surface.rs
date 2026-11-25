#[doc = include_str!("../../docs/api/contract/component_surface.doc/ChildrenSurface.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ChildrenSurface {
    /// Any child element is allowed (no restriction).
    #[default]
    Any,
    /// No children are allowed (a leaf position).
    None,
    /// Only components whose tag appears in this list are allowed.
    Only(Vec<&'static str>),
}

#[cfg(test)]
mod tests {
    use super::ChildrenSurface;

    #[test]
    fn default_children_surface_is_any() {
        assert_eq!(ChildrenSurface::default(), ChildrenSurface::Any);
    }
}
