use winit::window::Window;

use crate::control::{AppControl, SurfaceCapture};
use crate::geometry::Rect;
use crate::identity::{NodeId, WindowId};
use crate::render::frame::FrameSubmission;
use crate::runtime::UiRuntime;
use crate::system::capture::{CaptureProvider, CaptureRequest, CaptureResult, CaptureSource};

use super::VelloRenderer;

pub(super) fn capture_frame(
    control: &AppControl,
    window: Option<&Window>,
    runtime: &UiRuntime,
    renderer: &mut VelloRenderer,
    submission: &FrameSubmission,
    (width, height): (u32, u32),
) {
    let window = window.map(|window| WindowId::from_raw(window.id().into()));
    complete_captures(
        control,
        window,
        |node| runtime.presented_physical_bounds(node),
        || {
            let (width, height, pixels) = renderer
                .render_to_image(submission, width, height)
                .map_err(|err| format!("{err:?}").into_boxed_str())?;
            let mut capture = SurfaceCapture::default();
            if !capture.set_frame(width, height, pixels) {
                return Err("capture returned an invalid frame".into());
            }
            Ok(capture)
        },
    );
}

fn complete_captures(
    control: &AppControl,
    window: Option<WindowId>,
    node_bounds: impl Fn(NodeId) -> Option<Rect>,
    mut read_frame: impl FnMut() -> Result<SurfaceCapture, Box<str>>,
) {
    let requests = control.take_frame_captures();
    let mut frame = None;
    for (request, complete) in requests {
        let request = control.protect_capture_request(request);
        let request = match request.source() {
            CaptureSource::Surface | CaptureSource::Region(_) => request,
            CaptureSource::Window(id) if Some(*id) == window => request
                .protected_regions()
                .iter()
                .copied()
                .fold(CaptureRequest::surface(), CaptureRequest::protecting),
            CaptureSource::Node(id) => {
                let Some(bounds) = node_bounds(*id) else {
                    complete(CaptureResult::Failed("node has no presented bounds".into()));
                    continue;
                };
                request
                    .protected_regions()
                    .iter()
                    .copied()
                    .fold(CaptureRequest::region(bounds), CaptureRequest::protecting)
            }
            _ => {
                complete(CaptureResult::Unsupported);
                continue;
            }
        };
        let capture = frame.get_or_insert_with(&mut read_frame);
        let result = match capture {
            Ok(capture) => capture.capture(request),
            Err(reason) => CaptureResult::Failed(reason.clone()),
        };
        complete(result);
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;
    use crate::geometry::{Point, Rect, Size};

    #[test]
    fn matching_requests_share_one_readback_and_keep_separate_protection() {
        let control = AppControl::new();
        let window = WindowId::from_raw(1);
        let samples = Rc::new(RefCell::new(Vec::new()));
        let first_pixel = Rect::new(Point::ORIGIN, Size::new(1.0, 1.0));
        for request in [
            CaptureRequest::window(WindowId::from_raw(2)),
            CaptureRequest::window(window).protecting(first_pixel),
            CaptureRequest::region(first_pixel),
        ] {
            let samples = samples.clone();
            control.capture_next_frame(request, move |result| samples.borrow_mut().push(result));
        }
        let mut reads = 0;
        complete_captures(
            &control,
            Some(window),
            |_| None,
            || {
                reads += 1;
                let mut frame = SurfaceCapture::default();
                assert!(frame.set_frame(1, 1, vec![255, 0, 0, 255]));
                Ok(frame)
            },
        );
        let samples = samples.borrow();
        assert_eq!(reads, 1);
        assert_eq!(samples[0], CaptureResult::Unsupported);
        assert_eq!(samples[1].image().unwrap().bytes(), &[0, 0, 0, 0]);
        assert_eq!(samples[2].image().unwrap().bytes(), &[255, 0, 0, 255]);
    }

    #[test]
    fn queued_capture_keeps_old_protection_and_adds_current_protection() {
        let control = AppControl::new();
        let left = Rect::new(Point::ORIGIN, Size::new(1.0, 1.0));
        let right = Rect::new(Point::new(1.0, 0.0), Size::new(1.0, 1.0));
        control.set_capture_protected_regions(vec![left]);
        let result = Rc::new(RefCell::new(None));
        let destination = result.clone();
        control.capture_next_frame(CaptureRequest::surface(), move |capture| {
            *destination.borrow_mut() = Some(capture);
        });
        control.set_capture_protected_regions(vec![right]);
        complete_captures(
            &control,
            None,
            |_| None,
            || {
                let mut frame = SurfaceCapture::default();
                assert!(frame.set_frame(2, 1, vec![255, 0, 0, 255, 0, 255, 0, 255]));
                Ok(frame)
            },
        );
        assert_eq!(
            result.borrow().as_ref().unwrap().image().unwrap().bytes(),
            &[0; 8],
        );
    }

    #[test]
    fn node_capture_uses_presented_bounds_in_physical_pixels() {
        use crate::css::{
            compiler::StylesheetCompiler, parser::StylesheetParser, source::CssSource,
        };
        use crate::identity::{ComponentId, RuntimeId};
        use crate::view::view::{CLASS, View, ViewNode, ViewProperty, ViewValue};

        let size = Size::new(8.0, 4.0);
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), size);
        runtime.set_surface(size, 2.0);
        let view = View::single(
            ViewNode::element(ComponentId::from_raw(1))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("target".into()))),
        );
        let source =
            CssSource::anonymous(".target { width: 2px; height: 1px; transform: translateX(1px) }");
        let parsed = StylesheetParser::new(&source).parse();
        let sheet = StylesheetCompiler::new().compile(&parsed).matchable;
        let _ = runtime.render_now(&view, &[sheet]);
        let root = runtime.tree().root().unwrap();
        let node = runtime.tree().get(root).unwrap().child_nodes()[0];
        let control = AppControl::new();
        let result = Rc::new(RefCell::new(None));
        let destination = result.clone();
        control.capture_next_frame(CaptureRequest::node(node), move |capture| {
            *destination.borrow_mut() = Some(capture);
        });
        complete_captures(
            &control,
            None,
            |node| runtime.presented_physical_bounds(node),
            || {
                let mut frame = SurfaceCapture::default();
                let pixels: Vec<_> = (0..32u8).flat_map(|value| [value, 0, 0, 255]).collect();
                assert!(frame.set_frame(8, 4, pixels));
                Ok(frame)
            },
        );
        let result = result.borrow();
        let image = result.as_ref().unwrap().image().unwrap();
        assert_eq!(image.size(), Size::new(4.0, 2.0));
        let red: Vec<_> = image
            .bytes()
            .chunks_exact(4)
            .map(|pixel| pixel[0])
            .collect();
        assert_eq!(red, [2, 3, 4, 5, 10, 11, 12, 13]);
    }
}
