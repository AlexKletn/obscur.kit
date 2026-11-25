//! Write cached contours and glyphs to the frame mesh.

use super::super::{Mesh, Operation, Vertex, pixel_bounds, premultiply, vertex_position};
use super::MeshEmitter;
use crate::renderer::lyon::geometry::{ColorDraw, Contour, Geometry, Shape};
use obscur_geometry::{AffineTransform, Point, Rect};
use obscur_scene::color::Color;
use std::ops::Range;

impl MeshEmitter<'_> {
    pub(super) fn shape(
        &mut self,
        shape: Shape<'_>,
        color: Color,
        transform: AffineTransform,
    ) -> Result<Range<u32>, &'static str> {
        let Geometry::Mesh(contour) = self.geometry.get(shape, self.fonts)? else {
            unreachable!("path geometry is a contour");
        };
        emit(
            self.mesh,
            contour,
            color,
            transform,
            Point::new(0.0, 0.0),
            self.transform_slot,
            self.gradient,
            false,
        )
    }

    pub(super) fn draw_glyph(
        &mut self,
        shape: Shape<'_>,
        color: Color,
        transform: AffineTransform,
        offset: Point,
        depth: u32,
        scene_groups: usize,
    ) -> Result<(), &'static str> {
        let emit = |mesh: &mut Mesh, contour, paint, gradient, linear| {
            emit(
                mesh,
                contour,
                paint,
                transform,
                offset,
                self.transform_slot,
                gradient,
                linear,
            )
        };
        match self.geometry.get(shape, self.fonts)? {
            Geometry::Mesh(contour) => {
                let indices = emit(self.mesh, contour, color, self.gradient, false)?;
                self.mesh.draw(Operation::Paint, depth, indices);
                Ok(())
            }
            Geometry::Color {
                layers,
                linear_group,
            } => {
                let linear = *linear_group;
                let mut depth = depth;
                let mut clips = Vec::new();
                let mut groups = Vec::new();
                let mut composites = Vec::new();
                let vertex_start = self.mesh.vertices.len();
                if linear {
                    self.mesh.group_depth = self.mesh.group_depth.max(scene_groups + 1);
                    self.mesh.group_marker(Operation::BeginGroup);
                }
                for layer in layers {
                    let (operation, draw_depth, indices) = match layer {
                        ColorDraw::BeginGroup(blend) => {
                            groups.push(*blend);
                            self.mesh.group_depth =
                                self.mesh.group_depth.max(scene_groups + 1 + groups.len());
                            self.mesh.needs_blend |= blend.is_some();
                            self.mesh.group_marker(Operation::BeginGroup);
                            continue;
                        }
                        ColorDraw::EndGroup => {
                            let blend = groups.pop().expect("balanced COLRv1 groups");
                            composites.push(self.mesh.draws.len());
                            self.mesh.group_marker(Operation::EndGroup {
                                encode_srgb: false,
                                blend,
                                opacity: 1.0,
                                bounds: [0; 4],
                                color_filter: None,
                                blur_radius: 0.0,
                                stencil_depth: depth,
                            });
                            continue;
                        }
                        ColorDraw::Fill(paint, alpha, contour) => {
                            let gradient = if paint.is_none() { self.gradient } else { 0 };
                            let mut paint = paint.unwrap_or(color);
                            paint.a *= alpha;
                            let indices = emit(self.mesh, contour, paint, gradient, linear)?;
                            (Operation::Paint, depth, indices)
                        }
                        ColorDraw::Gradient(paint, contour) => {
                            if !self.mesh.retained_parameters {
                                return Err("COLRv1 gradient requires local vertex coordinates");
                            }
                            let foreground = if self.gradient == 0 {
                                color
                            } else {
                                self.mesh
                                    .gradients
                                    .first_color(self.gradient)
                                    .unwrap_or(Color::TRANSPARENT)
                            };
                            let gradient = self.mesh.gradients.insert_font(paint, foreground)?;
                            let indices = emit(self.mesh, contour, Color::WHITE, gradient, linear)?;
                            (Operation::Paint, depth, indices)
                        }
                        ColorDraw::Push(contour) => {
                            if depth >= 255 {
                                return Err("stencil nesting deeper than 255 levels");
                            }
                            let indices = emit(self.mesh, contour, Color::WHITE, 0, false)?;
                            clips.push(indices.clone());
                            depth += 1;
                            (Operation::Push, depth - 1, indices)
                        }
                        ColorDraw::Pop => {
                            let indices = clips.pop().expect("balanced COLRv1 clips");
                            depth -= 1;
                            (Operation::Pop, depth + 1, indices)
                        }
                    };
                    self.mesh.draw(operation, draw_depth, indices);
                }
                if linear {
                    // Destination-sensitive modes also touch pixels outside the source.
                    // The union of the glyph contours encloses both operands.
                    let glyph_bounds = pixel_bounds(
                        &self.mesh.vertices[vertex_start..],
                        &self.mesh.parameters,
                        self.extent,
                    );
                    for index in composites {
                        if let Operation::EndGroup { bounds, .. } =
                            &mut self.mesh.draws[index].operation
                        {
                            *bounds = glyph_bounds;
                        }
                    }
                    self.mesh.group_marker(Operation::EndGroup {
                        encode_srgb: true,
                        blend: None,
                        opacity: 1.0,
                        bounds: glyph_bounds,
                        color_filter: None,
                        blur_radius: 0.0,
                        stencil_depth: depth,
                    });
                }
                Ok(())
            }
            Geometry::Bitmap(bitmap) => {
                let bitmap = bitmap.clone();
                let tile = Rect::new(
                    Point::new(
                        bitmap.rect.origin.x + offset.x,
                        bitmap.rect.origin.y + offset.y,
                    ),
                    bitmap.rect.size,
                );
                if let Some((image, indices)) = self.image(
                    &bitmap.image,
                    tile,
                    tile,
                    [false; 2],
                    transform,
                    bitmap.shear,
                )? {
                    self.mesh.draw(Operation::Image(image), depth, indices);
                }
                Ok(())
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn emit(
    mesh: &mut Mesh,
    buffers: &Contour,
    color: Color,
    transform: AffineTransform,
    offset: Point,
    transform_slot: u32,
    gradient: u32,
    linear: bool,
) -> Result<Range<u32>, &'static str> {
    let base = u32::try_from(mesh.vertices.len()).map_err(|_| "too many vertices")?;
    let count = u32::try_from(buffers.vertices.len()).map_err(|_| "too many vertices")?;
    base.checked_add(count).ok_or("too many vertices")?;
    let start = u32::try_from(mesh.indices.len()).map_err(|_| "too many indices")?;
    let color = premultiply(color);
    for vertex in &buffers.vertices {
        mesh.vertices.push(Vertex {
            position: vertex_position(
                mesh.retained_parameters,
                transform,
                Point::new(vertex.x + offset.x, vertex.y + offset.y),
            )?,
            color,
            shadow: [offset.x, offset.y, 0.0, if linear { -1.0 } else { 0.0 }],
            shadow_parameters: [0.0; 2],
            transform: transform_slot,
            gradient: gradient,
        });
    }
    mesh.indices
        .extend(buffers.indices.iter().map(|index| base + index));
    let end = u32::try_from(mesh.indices.len()).map_err(|_| "too many indices")?;
    Ok(start..end)
}
