#![doc = include_str!("resolved_frame.doc/_module.md")]

use obscur_runtime::UiRuntime;
use obscur_runtime::frame::FrameResult;
use obscur_style::matching::Stylesheet;
use obscur_view::view::View;

#[doc = include_str!("resolved_frame.doc/render_resolved.md")]
pub fn render_resolved(runtime: &mut UiRuntime, view: &View, sheet: &Stylesheet) -> FrameResult {
    let _ = runtime.render_now(view, std::slice::from_ref(sheet));
    let _ = runtime.poll_resource_tasks();
    runtime.render_now(view, std::slice::from_ref(sheet))
}
