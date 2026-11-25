//! GPU resources for the experimental Sparse Strips renderer.

use obscur_render::renderer::RendererError;
use obscur_scene::command::DisplayList;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};

use crate::conversion::{SceneConverter, hybrid_scene};
use crate::error::VelloRenderError;

use super::VelloFrame;

#[derive(Default)]
pub(super) struct HybridRenderer {
    resources: Resources,
    scene: Option<((u16, u16), Scene)>,
    renderer: Option<Renderer>,
}

impl HybridRenderer {
    pub(super) fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &VelloFrame,
        list: &DisplayList,
        converter: &SceneConverter,
    ) -> Result<(), RendererError> {
        let (width, height) = frame.size();
        let size = u16::try_from(width)
            .and_then(|w| u16::try_from(height).map(|h| (w, h)))
            .map_err(|_| {
                VelloRenderError::RenderFailed("hybrid surface extent exceeds u16".into())
                    .into_renderer_error()
            })?;
        if self
            .scene
            .as_ref()
            .is_some_and(|(current, _)| *current != size)
        {
            self.scene = None;
        }
        let (_, scene) = self
            .scene
            .get_or_insert_with(|| (size, Scene::new(size.0, size.1)));
        let renderer = self.renderer.get_or_insert_with(|| {
            Renderer::new(
                device,
                &RenderTargetConfig {
                    width,
                    height,
                    format: frame.texture().format(),
                },
            )
        });
        scene.reset();
        hybrid_scene::emit(scene, &mut self.resources, list, converter);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("obscur-vello-hybrid"),
        });
        renderer
            .render(
                scene,
                &mut self.resources,
                device,
                queue,
                &mut encoder,
                &RenderSize { width, height },
                frame.view(),
                &TextureBindings::default(),
            )
            .map_err(|error| {
                VelloRenderError::RenderFailed(error.to_string().into()).into_renderer_error()
            })?;
        queue.submit([encoder.finish()]);
        Ok(())
    }
}
