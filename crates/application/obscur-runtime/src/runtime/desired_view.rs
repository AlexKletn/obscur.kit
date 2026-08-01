#![doc = include_str!("desired_view.doc/_module.md")]

use obscur_view::view::View;

#[doc = include_str!("desired_view.doc/DesiredView.md")]
pub(crate) enum DesiredView<'a> {
    #[doc = include_str!("desired_view.doc/Ready.md")]
    Ready(&'a View),
    #[doc = include_str!("desired_view.doc/Build.md")]
    Build(&'a mut dyn FnMut() -> View),
}
