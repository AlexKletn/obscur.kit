//! Segmented compositing: the render path a `backdrop-filter` forces (spec §5.2,
//! CSS `backdrop-filter`).

use obscur_render::renderer::RendererError;

use crate::conversion::SceneSegment;
use crate::effects::{ColorMatrix, GaussianBlur};
use crate::error::VelloRenderError;
use crate::renderer::filter_layer_pass::PreparedFilterLayers;
use crate::renderer::{VelloDevice, VelloFrame};

#[doc = include_str!("../../docs/api/renderer/segmented_compositor.doc/SegmentedCompositor.md")]
pub(crate) struct SegmentedCompositor<'a> {
    device: &'a mut VelloDevice,
    blur: &'a mut Option<GaussianBlur>,
    color_matrix: &'a mut Option<ColorMatrix>,
}

impl<'a> SegmentedCompositor<'a> {
    /// Borrows the device and the renderer's cached filter pipelines for the
    /// duration of one segmented frame.
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

    #[doc = include_str!("../../docs/api/renderer/segmented_compositor.doc/composite.md")]
    pub(crate) fn composite(
        &mut self,
        segments: Vec<SceneSegment>,
        width: u32,
        height: u32,
    ) -> Result<VelloFrame, RendererError> {
        let gpu_device = self.device.device().clone();
        let gpu_queue = self.device.queue().clone();

        // The accumulating frame `T`: segments composite onto it in order, and each
        // backdrop boundary frosts it in place. Clear it before the first segment.
        let frame = VelloFrame::new_offscreen(&gpu_device, width, height);
        let (width, height) = frame.size();
        {
            let mut encoder = gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("obscur-vello-backdrop-clear"),
            });
            clear_frame(&mut encoder, frame.view());
            gpu_queue.submit(std::iter::once(encoder.finish()));
        }

        // Build (once) / reuse the owned blur + composite pipelines.
        if self.blur.is_none() {
            *self.blur = Some(GaussianBlur::new(&gpu_device));
        }

        let mut scratch_frame = None;
        let mut has_content = false;
        let params = vello::RenderParams {
            base_color: vello::peniko::Color::TRANSPARENT,
            width,
            height,
            antialiasing_method: vello::AaConfig::Area,
        };
        for segment in segments {
            match segment {
                SceneSegment::Content { scene, blurs } => {
                    if !scene.encoding().is_empty() {
                        let scratch = scratch_frame.get_or_insert_with(|| {
                            VelloFrame::new_offscreen(&gpu_device, width, height)
                        });
                        let renderer = self.device.renderer_mut();
                        let result = if has_content {
                            // Vello sees prior pixels when applying blend and
                            // composition operators to this segment.
                            let mut backdrop = renderer.register_texture(frame.texture().clone());
                            backdrop.alpha_type = vello::peniko::ImageAlphaType::AlphaPremultiplied;
                            let mut seeded = vello::Scene::new();
                            seeded.draw_image(&backdrop, vello::kurbo::Affine::IDENTITY);
                            seeded.append(&scene, None);
                            let result = renderer.render_to_texture(
                                &gpu_device,
                                &gpu_queue,
                                &seeded,
                                scratch.view(),
                                &params,
                            );
                            renderer.unregister_texture(backdrop);
                            result
                        } else {
                            renderer.render_to_texture(
                                &gpu_device,
                                &gpu_queue,
                                &scene,
                                scratch.view(),
                                &params,
                            )
                        };
                        result.map_err(|error| {
                            VelloRenderError::RenderFailed(error.to_string().into())
                                .into_renderer_error()
                        })?;

                        // Vello writes straight alpha. Replace the accumulator
                        // with the premultiplied form before applying filters.
                        let mut encoder =
                            gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("obscur-vello-segment-replace"),
                            });
                        clear_frame(&mut encoder, frame.view());
                        let gaussian = self.blur.as_ref().expect("blur pipelines built above");
                        gaussian.composite_over(
                            &gpu_device,
                            &mut encoder,
                            scratch.view(),
                            frame.view(),
                        );
                        gpu_queue.submit(std::iter::once(encoder.finish()));
                        has_content = true;
                    }
                    if blurs.is_empty() {
                        continue;
                    }
                    has_content = true;
                    // Render this segment's `filter: blur` sub-scenes into their own
                    // margin-padded offscreens (same prep as the fast path).
                    let prepared = PreparedFilterLayers::render(
                        &gpu_device,
                        &gpu_queue,
                        self.device.renderer_mut(),
                        blurs,
                    )?;

                    // Composite filter output before the next content segment.
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
                    blur,
                    coverage,
                    ..
                } => {
                    if !has_content || coverage.encoding().is_empty() {
                        continue;
                    }
                    let mask = scratch_frame.get_or_insert_with(|| {
                        VelloFrame::new_offscreen(&gpu_device, width, height)
                    });
                    self.device
                        .renderer_mut()
                        .render_to_texture(&gpu_device, &gpu_queue, &coverage, mask.view(), &params)
                        .map_err(|error| {
                            VelloRenderError::RenderFailed(error.to_string().into())
                                .into_renderer_error()
                        })?;
                    let gaussian = self.blur.as_ref().expect("blur pipelines built above");
                    let mut encoder =
                        gpu_device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("obscur-vello-backdrop"),
                        });
                    gaussian.render_masked_backdrop(
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
                        blur,
                        mask.view(),
                    );
                    gpu_queue.submit(std::iter::once(encoder.finish()));
                }
            }
        }

        Ok(frame)
    }
}

fn clear_frame(encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
    drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("obscur-vello-backdrop-clear-pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: view,
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
}
