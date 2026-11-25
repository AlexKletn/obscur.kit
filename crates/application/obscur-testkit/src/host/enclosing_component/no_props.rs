use obscur_view::component::{FromProps, PropsError};
use obscur_view::view::ViewProperty;

/// Props for a component that takes none: it decodes from anything.
pub(super) struct NoProps;

impl FromProps for NoProps {
    fn from_props(_: &[ViewProperty]) -> Result<Self, PropsError> {
        Ok(Self)
    }
}
