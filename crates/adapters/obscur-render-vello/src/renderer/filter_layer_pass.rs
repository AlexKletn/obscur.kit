//! The `filter` layer pass: the sub-scenes the converter peeled off the main
//! scene, rendered into their own offscreens and composited back through the
//! adapter's owned wgpu passes (spec §5.2).

use obscur_render::renderer::RendererError;
use vello::Scene;
use vello::kurbo::Affine;

use crate::conversion::BlurLayer;
use crate::effects::{ColorMatrix, GaussianBlur};
use crate::error::VelloRenderError;
use crate::renderer::VelloFrame;

#[doc = include_str!("../../docs/api/renderer/filter_layer_pass.doc/PreparedFilterLayers.md")]
pub(crate) struct PreparedFilterLayers {
    layers: Vec<(VelloFrame, (f32, f32), BlurLayer)>,
}

impl PreparedFilterLayers {
    #[doc = include_str!("../../docs/api/renderer/filter_layer_pass.doc/render.md")]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub(crate) fn render(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vello: &mut vello::Renderer,
        layers: Vec<BlurLayer>,
    ) -> Result<Self, RendererError> {
        let mut prepared: Vec<(VelloFrame, (f32, f32), BlurLayer)> =
            Vec::with_capacity(layers.len());
        for blur in layers {
            let margin = (blur.radius * 3.0).ceil().max(1.0);
            let s_width = (blur.bounds.size.width + 2.0 * margin).ceil().max(1.0) as u32;
            let s_height = (blur.bounds.size.height + 2.0 * margin).ceil().max(1.0) as u32;
            let s_frame = VelloFrame::new_offscreen(device, s_width, s_height);
            let (s_width, s_height) = s_frame.size();
            // Offset the sub-scene so the bounds' top-left maps to (margin, margin).
            let offset = Affine::translate((
                f64::from(margin) - f64::from(blur.bounds.origin.x),
                f64::from(margin) - f64::from(blur.bounds.origin.y),
            ));
            let mut placed = Scene::new();
            placed.append(&blur.scene, Some(offset));
            let sub_params = vello::RenderParams {
                base_color: vello::peniko::Color::TRANSPARENT,
                width: s_width,
                height: s_height,
                antialiasing_method: vello::AaConfig::Area,
            };
            vello
                .render_to_texture(device, queue, &placed, s_frame.view(), &sub_params)
                .map_err(|error| {
                    VelloRenderError::RenderFailed(error.to_string().into()).into_renderer_error()
                })?;
            // Absolute pixel top-left the padded offscreen composites back at.
            let origin = (blur.bounds.origin.x - margin, blur.bounds.origin.y - margin);
            prepared.push((s_frame, origin, blur));
        }
        Ok(Self { layers: prepared })
    }

    /// Whether any of these layers carries a colour matrix — the question that
    /// decides whether the [`ColorMatrix`] pipeline is built at all, so a scene
    /// with only blurs stays byte-identical to the pre-colour-op path.
    pub(crate) fn needs_color_matrix(&self) -> bool {
        self.layers
            .iter()
            .any(|(_, _, layer)| layer.color_matrix.is_some())
    }

    #[doc = include_str!("../../docs/api/renderer/filter_layer_pass.doc/composite.md")]
    pub(crate) fn composite(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        gaussian: &GaussianBlur,
        color_matrix: Option<&ColorMatrix>,
        target_view: &wgpu::TextureView,
        target_size: (u32, u32),
    ) -> Vec<wgpu::Texture> {
        let mut scratch: Vec<wgpu::Texture> = Vec::new();
        for (s_frame, origin, blur) in &self.layers {
            if let Some(texture) = apply_filter_layer(
                device,
                encoder,
                gaussian,
                color_matrix,
                s_frame,
                *origin,
                blur,
                target_view,
                target_size,
            ) {
                scratch.push(texture);
            }
        }
        scratch
    }
}

#[doc = include_str!("../../docs/api/renderer/filter_layer_pass.doc/apply_filter_layer.md")]
#[allow(clippy::too_many_arguments)]
fn apply_filter_layer(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    gaussian: &GaussianBlur,
    color_matrix: Option<&ColorMatrix>,
    s_frame: &VelloFrame,
    origin: (f32, f32),
    layer: &BlurLayer,
    target_view: &wgpu::TextureView,
    target_size: (u32, u32),
) -> Option<wgpu::Texture> {
    let (sw, sh) = s_frame.size();

    let color_matrix_scratch = layer.color_matrix.and_then(|matrix| {
        let color_matrix = color_matrix?;
        let transformed = color_matrix.scratch_texture(device, sw, sh);
        let sub_scene_view = s_frame
            .texture()
            .create_view(&wgpu::TextureViewDescriptor::default());
        let transformed_view = transformed.create_view(&wgpu::TextureViewDescriptor::default());
        color_matrix.apply(device, encoder, &sub_scene_view, &transformed_view, &matrix);
        Some(transformed)
    });

    // Both Vello and the color-matrix pass produce straight RGBA.
    let source_view = color_matrix_scratch
        .as_ref()
        .unwrap_or_else(|| s_frame.texture())
        .create_view(&wgpu::TextureViewDescriptor::default());

    if layer.radius > 0.0 {
        gaussian.render_view(
            device,
            encoder,
            &source_view,
            (sw, sh),
            origin,
            layer.radius,
            target_view,
            target_size,
        );
    } else {
        gaussian.composite_texture(
            device,
            encoder,
            &source_view,
            (sw, sh),
            origin,
            target_view,
            target_size,
            true,
        );
    }

    color_matrix_scratch
}
