//! Converts display commands into indexed draws and stencil operations.

use std::ops::Range;

use obscur_geometry::{AffineTransform, Point};
use obscur_scene::clip::{Clip, ClipShape};
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::paint::{Paint, SceneImage};

use crate::conversion::SceneConverter;

#[path = "mesh_emitter.rs"]
mod mesh_emitter;
use mesh_emitter::contains_one_fill;
#[path = "mesh_rect.rs"]
mod mesh_rect;
use mesh_emitter::MeshEmitter;
pub(super) use mesh_rect::path_rect;

pub(super) const MAX_DRAW_PARAMETERS: usize = 256;
pub(super) const MAX_COLOR_FILTERS: usize = 128;

pub(super) use super::draw::Draw;
pub(super) use super::draw_parameters::DrawParameters;
pub(super) use super::operation::Operation;
pub(super) use super::vertex::Vertex;

use super::geometry::GeometryCache;
use super::gradient::Gradients;

#[derive(Default)]
pub(super) struct Mesh {
    pub(super) vertices: Vec<Vertex>,
    pub(super) indices: Vec<u32>,
    pub(super) draws: Vec<Draw>,
    pub(super) group_depth: usize,
    pub(super) color_filters: Vec<[f32; 24]>,
    pub(super) blur_count: usize,
    pub(super) needs_backdrop: bool,
    pub(super) needs_blend: bool,
    pub(super) parameters: Vec<DrawParameters>,
    retained_parameters: bool,
    rect_vertices: Vec<Range<usize>>,
    pub(super) text_vertices: Vec<Range<usize>>,
    pub(super) gradients: Gradients,
    pub(super) images: Vec<SceneImage>,
}

impl Mesh {
    pub(super) fn lower(
        &mut self,
        list: &DisplayList,
        extent: (u32, u32),
        geometry: &mut GeometryCache,
        fonts: &SceneConverter,
    ) -> Result<(), &'static str> {
        self.retained_parameters = list
            .commands()
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    PaintCommand::PushTransform(_) | PaintCommand::PushLayer(_)
                )
            })
            .take(MAX_DRAW_PARAMETERS)
            .count()
            < MAX_DRAW_PARAMETERS;
        self.parameters.clear();
        self.push_parameters(AffineTransform::IDENTITY, 1.0);
        self.vertices.clear();
        self.rect_vertices.clear();
        self.rect_vertices.resize(list.len(), 0..0);
        self.text_vertices.clear();
        if crate::renderer::damage_stats::DamageStats::scene_enabled() {
            self.text_vertices.resize(list.len(), 0..0);
        }
        self.gradients.clear();
        self.images.clear();
        self.indices.clear();
        self.draws.clear();
        self.group_depth = 0;
        self.color_filters.clear();
        self.blur_count = 0;
        self.needs_backdrop = false;
        self.needs_blend = false;
        geometry.begin();
        MeshEmitter::new(self, extent, geometry, fonts).emit(list)?;
        geometry.finish();
        Ok(())
    }

    fn push_parameters(&mut self, transform: AffineTransform, opacity: f32) -> u32 {
        let slot = u32::try_from(self.parameters.len()).expect("bounded draw parameters");
        self.parameters.push(DrawParameters { transform, opacity });
        slot
    }

    pub(super) fn update_parameters(
        &mut self,
        previous: &DisplayList,
        next: &DisplayList,
        geometry_changed: &mut bool,
    ) -> bool {
        if !self.retained_parameters
            || self.group_depth != 0
            || self.needs_backdrop
            || previous.len() != next.len()
        {
            return false;
        }
        let commands = next.commands();
        let mut stack = Vec::new();
        let mut slot = 1;
        let mut single_fill_opacity = None;
        for (index, (old, new)) in previous.iter().zip(commands).enumerate() {
            let transform = stack.last().copied().unwrap_or(AffineTransform::IDENTITY);
            match (old, new) {
                (PaintCommand::PushTransform(_), PaintCommand::PushTransform(next)) => {
                    let transform = transform.multiply(*next);
                    if !transform.coeffs.iter().all(|value| value.is_finite()) {
                        return false;
                    }
                    self.parameters[slot] = DrawParameters {
                        transform,
                        opacity: 1.0,
                    };
                    slot += 1;
                    stack.push(transform);
                }
                (_, PaintCommand::PopTransform | PaintCommand::PopLayer) if old == new => {
                    stack.pop();
                }
                (PaintCommand::PushLayer(old_layer), PaintCommand::PushLayer(layer)) => {
                    let single_fill =
                        !layer.is_isolated() && contains_one_fill(&commands[index + 1..]);
                    if old != new
                        && (!single_fill
                            || old_layer.transform() != layer.transform()
                            || old_layer.clip() != layer.clip()
                            || old_layer.is_isolated() != layer.is_isolated()
                            || old_layer.blend_mode() != layer.blend_mode()
                            || old_layer.composition() != layer.composition()
                            || old_layer.effects() != layer.effects()
                            || !(0.0..=1.0).contains(&layer.opacity()))
                    {
                        return false;
                    }
                    let transform = if layer.transform() == AffineTransform::IDENTITY {
                        transform
                    } else {
                        let transform = transform.multiply(layer.transform());
                        if !transform.coeffs.iter().all(|value| value.is_finite()) {
                            return false;
                        }
                        self.parameters[slot] = DrawParameters {
                            transform,
                            opacity: 1.0,
                        };
                        slot += 1;
                        transform
                    };
                    stack.push(transform);
                    if single_fill {
                        single_fill_opacity = Some(layer.opacity());
                    }
                }
                (
                    PaintCommand::PushClip(Clip::Shape(ClipShape::Rect(old))),
                    PaintCommand::PushClip(Clip::Shape(ClipShape::Rect(new))),
                ) if old != new => {
                    if !self.update_rect(index, *old, *new, transform) {
                        return false;
                    }
                    *geometry_changed = true;
                }
                (
                    PaintCommand::DrawPath {
                        path: old_path,
                        paint: Paint::Fill(old_brush),
                    },
                    PaintCommand::DrawPath {
                        path,
                        paint: Paint::Fill(brush),
                    },
                ) if old != new => {
                    if !self.update_rect_fill(
                        index,
                        (old_path, old_brush),
                        (path, brush),
                        transform,
                    ) {
                        return false;
                    }
                    *geometry_changed = true;
                    if let Some(opacity) = single_fill_opacity.take() {
                        self.parameters[slot] = DrawParameters { transform, opacity };
                        slot += 1;
                    }
                }
                (_, PaintCommand::DrawPath { .. }) if old == new => {
                    if let Some(opacity) = single_fill_opacity.take() {
                        self.parameters[slot] = DrawParameters { transform, opacity };
                        slot += 1;
                    }
                }
                _ if old == new => {}
                _ => return false,
            }
        }
        true
    }

    fn group_marker(&mut self, operation: Operation) {
        self.draws.push(Draw {
            operation,
            depth: 0,
            indices: 0..0,
        });
    }

    fn draw(&mut self, operation: Operation, depth: u32, indices: Range<u32>) {
        if indices.is_empty() {
            return;
        }
        if matches!(operation, Operation::Paint | Operation::Image(_))
            && let Some(last) = self.draws.last_mut()
            && last.operation == operation
            && last.depth == depth
            && last.indices.end == indices.start
        {
            last.indices.end = indices.end;
        } else {
            self.draws.push(Draw {
                operation,
                depth,
                indices,
            });
        }
    }
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub(super) fn pixel_bounds(
    vertices: &[Vertex],
    parameters: &[DrawParameters],
    (width, height): (u32, u32),
) -> [u32; 4] {
    if vertices.is_empty() {
        return [0; 4];
    }
    let mut lower = [width as f32, height as f32];
    let mut upper = [0.0_f32; 2];
    for vertex in vertices {
        let point = parameters[vertex.transform as usize]
            .transform
            .apply(Point::new(vertex.position[0], vertex.position[1]));
        let (x, y) = (point.x, point.y);
        lower[0] = lower[0].min(x);
        lower[1] = lower[1].min(y);
        upper[0] = upper[0].max(x);
        upper[1] = upper[1].max(y);
    }
    // The shader projection can round differently at integer clip edges.
    let x = (lower[0].floor() as u32).saturating_sub(1).min(width);
    let y = (lower[1].floor() as u32).saturating_sub(1).min(height);
    let right = (upper[0].ceil() as u32).saturating_add(1).min(width);
    let bottom = (upper[1].ceil() as u32).saturating_add(1).min(height);
    [x, y, right.saturating_sub(x), bottom.saturating_sub(y)]
}

fn premultiply(color: Color) -> [f32; 4] {
    let alpha = color.a.clamp(0.0, 1.0);
    [
        color.r.clamp(0.0, 1.0) * alpha,
        color.g.clamp(0.0, 1.0) * alpha,
        color.b.clamp(0.0, 1.0) * alpha,
        alpha,
    ]
}

fn vertex_position(
    local: bool,
    transform: AffineTransform,
    point: Point,
) -> Result<[f32; 2], &'static str> {
    let transformed = transform.apply(point);
    if !transformed.x.is_finite() || !transformed.y.is_finite() {
        return Err("non-finite device position");
    }
    let point = if local { point } else { transformed };
    Ok([point.x, point.y])
}
