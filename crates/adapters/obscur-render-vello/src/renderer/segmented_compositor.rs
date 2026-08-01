#![doc = include_str!("segmented_compositor.doc/_module.md")]

use obscur_render::renderer::RendererError;

use crate::conversion::SceneSegment;
use crate::effects::{ColorMatrix, GaussianBlur};
use crate::error::VelloRenderError;
use crate::renderer::filter_layer_pass::PreparedFilterLayers;
use crate::renderer::{VelloDevice, VelloFrame};

#[doc = include_str!("segmented_compositor.doc/SegmentedCompositor.md")]
pub(crate) struct SegmentedCompositor<'a> {
    device: &'a mut VelloDevice,
    blur: &'a mut Option<GaussianBlur>,
    color_matrix: &'a mut Option<ColorMatrix>,
}

impl<'a> SegmentedCompositor<'a> {
    #[doc = include_str!("segmented_compositor.doc/new.md")]
    pub(crate) fn new(
        device: &'a mut VelloDevice,
        blur: &'a mut Option<GaussianBlur>,
        color_matrix: &'a mut Option<ColorMatrix>,
    ) -> Self {
        Self {
            device,
            blur,
            color_matrix,
        }
    }

    #[doc = include_str!("segmented_compositor.doc/composite.md")]
    pub(crate) fn composite(
        &mut self,
        segments: Vec<SceneSegment>,
        width: u32,
        height: u32,
    ) -> Result<VelloFrame, RendererError> {
        let gpu_device = self.device.device().clone();
        let gpu_queue = self.device.queue().clone();

        // The accumulating frame `T`: segments composite onto it in order, and each
        // backdrop boundary frosts it in place. Clear it to transparent first (it is
        // never a Vello render target, so nothing else initialises it).
        let frame = VelloFrame::new_offscreen(&gpu_device, width, height);
        let (width, height) = frame.size();
        {
            let mut encoder = gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("obscur-vello-backdrop-clear"),
            });
            drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("obscur-vello-backdrop-clear-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: frame.view(),
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            }));
            gpu_queue.submit(std::iter::once(encoder.finish()));
        }

        // Build (once) / reuse the owned blur + composite pipelines.
        if self.blur.is_none() {
            *self.blur = Some(GaussianBlur::new(&gpu_device));
        }

        for segment in segments {
            match segment {
                SceneSegment::Content { scene, blurs } => {
                    // Render this segment's content into a fresh transparent
                    // offscreen (Vello clears it to `base_color`).
                    let seg_frame = VelloFrame::new_offscreen(&gpu_device, width, height);
                    let params = vello::RenderParams {
                        base_color: vello::peniko::Color::TRANSPARENT,
                        width,
                        height,
                        antialiasing_method: vello::AaConfig::Area,
                    };
                    self.device
                        .renderer_mut()
                        .render_to_texture(
                            &gpu_device,
                            &gpu_queue,
                            &scene,
                            seg_frame.view(),
                            &params,
                        )
                        .map_err(|error| {
                            VelloRenderError::RenderFailed(error.to_string().into())
                                .into_renderer_error()
                        })?;

                    // Render this segment's `filter: blur` sub-scenes into their own
                    // margin-padded offscreens (same prep as the fast path).
                    let prepared = PreparedFilterLayers::render(
                        &gpu_device,
                        &gpu_queue,
                        self.device.renderer_mut(),
                        blurs,
                    )?;

                    // Composite the segment (whole-frame) onto `T`, then its filter
                    // layers on top, in one encoder submitted after the Vello renders.
                    let needs_color = prepared.needs_color_matrix();
                    let gaussian = self.blur.as_ref().expect("blur pipelines built above");
                    // Build/reuse the color-matrix pipeline only when a color op is
                    // present (byte-identical no-color path otherwise).
                    let color_matrix = needs_color.then(|| {
                        self.color_matrix
                            .get_or_insert_with(|| ColorMatrix::new(&gpu_device))
                    });
                    let mut encoder =
                        gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("obscur-vello-segment-composite"),
                        });
                    gaussian.composite_over(
                        &gpu_device,
                        &mut encoder,
                        seg_frame.view(),
                        frame.view(),
                    );
                    let scratch = prepared.composite(
                        &gpu_device,
                        &mut encoder,
                        gaussian,
                        color_matrix.as_deref(),
                        frame.view(),
                        (width, height),
                    );
                    gpu_queue.submit(std::iter::once(encoder.finish()));
                    drop(scratch);
                }
                SceneSegment::Backdrop {
                    bounds,
                    radii,
                    blur,
                } => {
                    let gaussian = self.blur.as_ref().expect("blur pipelines built above");
                    let mut encoder =
                        gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("obscur-vello-backdrop"),
                        });
                    gaussian.render_backdrop(
                        &gpu_device,
                        &mut encoder,
                        frame.texture(),
                        frame.view(),
                        (width, height),
                        [
                            bounds.origin.x,
                            bounds.origin.y,
                            bounds.origin.x + bounds.size.width,
                            bounds.origin.y + bounds.size.height,
                        ],
                        // The backdrop mask's WGSL SDF (`rounded_rect_coverage` in
                        // `gaussian_blur.wgsl`) is a per-corner *circular* rounded
                        // box: it selects one corner radius by quadrant and does
                        // `length(q) - r`, which has no elliptical form. So a panel
                        // with elliptical corners (`border-radius: 50%` on a
                        // non-square box) frosts to the closest circle per corner —
                        // the mean of its two axes, exact whenever the corner is
                        // already circular. Confined to the backdrop-filter effect;
                        // the panel's own clip and fill keep the true ellipse via
                        // `Path`. See `docs/conformance-audit.md` finding 4-A.
                        [
                            radii.top_left.circular_approximation(),
                            radii.top_right.circular_approximation(),
                            radii.bottom_right.circular_approximation(),
                            radii.bottom_left.circular_approximation(),
                        ],
                        blur,
                    );
                    gpu_queue.submit(std::iter::once(encoder.finish()));
                }
            }
        }

        Ok(frame)
    }
}
