use std::sync::Arc;

use obscur_scene::{color::AlphaMode, paint::SceneImage};

use crate::geometry::{Rect, Size};
use crate::system::capture::{
    CaptureProvider, CaptureRequest, CaptureResult, CaptureSource, CapturedImage,
};

const RGBA8_BYTES_PER_PIXEL: usize = 4;

/// Captures an application-owned RGBA8 frame. Region coordinates are physical
/// pixels relative to the frame origin. Released pixels are premultiplied RGBA.
#[derive(Default)]
pub struct SurfaceCapture {
    frame: Option<SceneImage>,
}

impl SurfaceCapture {
    /// Replaces the source frame. Invalid data clears the previous frame.
    pub fn set_frame(&mut self, width: u32, height: u32, pixels: impl Into<Arc<[u8]>>) -> bool {
        self.frame = if width == 0 || height == 0 {
            None
        } else {
            SceneImage::new(width, height, pixels.into(), AlphaMode::Premultiplied)
        };
        self.frame.is_some()
    }

    /// Discards the source frame.
    pub fn clear(&mut self) {
        self.frame = None;
    }
}

impl CaptureProvider for SurfaceCapture {
    fn can_capture(&self, source: &CaptureSource) -> bool {
        matches!(source, CaptureSource::Surface | CaptureSource::Region(_))
    }

    fn protects_regions(&self, source: &CaptureSource) -> bool {
        self.can_capture(source)
    }

    fn capture(&mut self, request: CaptureRequest) -> CaptureResult {
        if !self.can_capture(request.source()) {
            return CaptureResult::Unsupported;
        }
        let Some(frame) = &self.frame else {
            return CaptureResult::Failed("no rendered frame is available".into());
        };
        let bounds_for = |rect| pixel_bounds(rect, frame.width(), frame.height());
        let [left, top, right, bottom] = match request.source() {
            CaptureSource::Region(rect) => match bounds_for(*rect) {
                Some(bounds) if bounds[0] < bounds[2] && bounds[1] < bounds[3] => bounds,
                _ => return CaptureResult::Failed("capture region has no pixels".into()),
            },
            _ => [0, 0, frame.width() as usize, frame.height() as usize],
        };
        let Some(protected) = request
            .protected_regions()
            .iter()
            .copied()
            .map(bounds_for)
            .collect::<Option<Vec<_>>>()
        else {
            return CaptureResult::Denied;
        };

        let width = right - left;
        let height = bottom - top;
        // SceneImage validates the full frame, and all crop edges are clamped to it.
        let stride_bytes = frame.width() as usize * RGBA8_BYTES_PER_PIXEL;
        let mut pixels = Vec::with_capacity(width * height * RGBA8_BYTES_PER_PIXEL);
        for y in top..bottom {
            let row_start = pixels.len();
            let source_row = y * stride_bytes;
            let source_start = source_row + left * RGBA8_BYTES_PER_PIXEL;
            let source_end = source_row + right * RGBA8_BYTES_PER_PIXEL;
            pixels.extend_from_slice(&frame.pixels()[source_start..source_end]);
            for &[px0, py0, px1, py1] in &protected {
                let start = left.max(px0);
                let end = right.min(px1);
                if y >= py0 && y < py1 && start < end {
                    let start_byte = row_start + (start - left) * RGBA8_BYTES_PER_PIXEL;
                    let end_byte = row_start + (end - left) * RGBA8_BYTES_PER_PIXEL;
                    pixels[start_byte..end_byte].fill(0);
                }
            }
        }
        CaptureResult::Captured(
            CapturedImage::new(
                Size::new(width as f32, height as f32),
                "rgba8-premultiplied",
            )
            .with_bytes(pixels),
        )
    }
}

fn pixel_bounds(rect: Rect, width_px: u32, height_px: u32) -> Option<[usize; 4]> {
    let x = f64::from(rect.origin.x);
    let y = f64::from(rect.origin.y);
    let width = f64::from(rect.size.width);
    let height = f64::from(rect.size.height);
    if ![x, y, width, height].iter().all(|value| value.is_finite()) || width < 0.0 || height < 0.0 {
        return None;
    }
    // All inputs are integral after rounding and bounded by the validated frame.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let edge = |value: f64, limit| value.clamp(0.0, f64::from(limit)) as usize;
    let left = edge(x.floor(), width_px);
    let top = edge(y.floor(), height_px);
    let right = if width == 0.0 {
        left
    } else {
        edge((x + width).ceil(), width_px)
    };
    let bottom = if height == 0.0 {
        top
    } else {
        edge((y + height).ceil(), height_px)
    };
    Some([left, top, right, bottom])
}

#[cfg(all(test, feature = "render-cpu"))]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use obscur_render::target::{BitmapTarget, RenderTarget};
    use obscur_render::{
        Renderer, RendererId,
        frame::{FrameSubmission, RenderFrame},
    };
    use obscur_render_cpu::renderer::CpuRenderer;
    use obscur_scene::{
        color::Color,
        command::{DisplayList, PaintCommand},
        paint::Paint,
        path::Path,
        scene::{SceneGeneration, SceneSnapshot},
    };

    use super::*;
    use crate::{control::AppControl, geometry::Point};

    #[test]
    fn app_capture_crops_rendered_pixels_and_redacts_fractional_protection() {
        let size = Size::new(4.0, 2.0);
        let list = DisplayList::from_commands(vec![PaintCommand::DrawPath {
            path: Path::rect(Rect::new(Point::ORIGIN, size)),
            paint: Paint::solid_fill(Color::rgba(1.0, 0.0, 0.0, 1.0)),
        }]);
        let submission = FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(list, SceneGeneration::from_raw(1)),
            size,
        ));
        let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
        renderer
            .render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
            .unwrap();
        let pixmap = renderer.pixmap().unwrap();

        let capture = Rc::new(RefCell::new(SurfaceCapture::default()));
        assert!(
            capture
                .borrow_mut()
                .set_frame(pixmap.width(), pixmap.height(), pixmap.data())
        );
        let control = AppControl::new();
        control.set_capture_provider(Some(capture.clone()));
        let crop = Rect::new(Point::new(1.0, 0.0), Size::new(2.0, 2.0));
        let secret = Rect::new(Point::new(2.25, 0.1), Size::new(0.25, 0.5));
        let result = control.capture(CaptureRequest::region(crop).protecting(secret));
        let image = result.image().unwrap();
        assert_eq!(image.size(), Size::new(2.0, 2.0));
        assert_eq!(
            image.bytes(),
            &[255, 0, 0, 255, 0, 0, 0, 0, 255, 0, 0, 255, 255, 0, 0, 255]
        );
        assert_eq!(pixmap.pixel(2, 0).unwrap().red(), 255);

        let invalid = Rect::new(Point::new(f32::NAN, 0.0), crop.size);
        assert_eq!(
            control.capture(CaptureRequest::surface().protecting(invalid)),
            CaptureResult::Denied
        );
        assert!(!capture.borrow_mut().set_frame(4, 2, vec![0; 4]));
        assert!(matches!(
            control.capture(CaptureRequest::surface()),
            CaptureResult::Failed(_)
        ));
    }
}
