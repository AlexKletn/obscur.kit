use std::ops::Range;

use obscur_geometry::{AffineTransform, CornerRadii, Point, Rect, Size};
use obscur_scene::clip::{Clip, ClipShape};
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::layer::{CompositionMode, LayerEffect};
use obscur_scene::paint::{BlendMode, Brush, Paint, SceneImage};
use obscur_scene::path::Path;

use crate::conversion::{SceneConverter, compose, mix};
use crate::renderer::lyon::geometry::{GeometryCache, Shape, StrokeGeometry};

use super::{
    MAX_COLOR_FILTERS, Mesh, Operation, Vertex, pixel_bounds, premultiply, vertex_position,
};

#[path = "mesh_geometry.rs"]
mod mesh_geometry;
#[path = "mesh_mask.rs"]
mod mesh_mask;
#[path = "paint_group.rs"]
mod paint_group;
use mesh_mask::ImageMask;
use paint_group::PaintGroup;

pub(super) fn contains_one_fill(commands: &[PaintCommand]) -> bool {
    let mut seen_fill = false;
    for command in commands {
        match command {
            PaintCommand::DrawPath {
                paint: Paint::Fill(_),
                ..
            } if !seen_fill => seen_fill = true,
            PaintCommand::PushClip(_)
            | PaintCommand::PopClip
            | PaintCommand::PushTransform(_)
            | PaintCommand::PopTransform => {}
            PaintCommand::PopLayer => return seen_fill,
            _ => return false,
        }
    }
    false
}

fn device_blur_radius(radius: f32, transform: AffineTransform) -> Result<f32, &'static str> {
    let [a, b, c, d, ..] = transform.coeffs;
    let scaled = radius * (a * d - b * c).abs().sqrt();
    if !scaled.is_finite() || !(0.0..=f32::from(u16::MAX)).contains(&scaled) {
        return Err("unsupported Lyon blur radius");
    }
    Ok(scaled)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn padded_bounds([x, y, width, height]: [u32; 4], radius: f32, (w, h): (u32, u32)) -> [u32; 4] {
    if width == 0 || height == 0 {
        return [0; 4];
    }
    let pad = ((3.0 * radius).ceil() as u32).saturating_add(1);
    let left = x.saturating_sub(pad).min(w);
    let top = y.saturating_sub(pad).min(h);
    [
        left,
        top,
        x.saturating_add(width).saturating_add(pad).min(w) - left,
        y.saturating_add(height).saturating_add(pad).min(h) - top,
    ]
}

pub(super) struct MeshEmitter<'a> {
    mesh: &'a mut Mesh,
    extent: (u32, u32),
    geometry: &'a mut GeometryCache,
    fonts: &'a SceneConverter,
    transform_slot: u32,
    gradient: u32,
}

impl<'a> MeshEmitter<'a> {
    pub(super) fn new(
        mesh: &'a mut Mesh,
        extent: (u32, u32),
        geometry: &'a mut GeometryCache,
        fonts: &'a SceneConverter,
    ) -> Self {
        Self {
            mesh,
            extent,
            geometry,
            fonts,
            transform_slot: 0,
            gradient: 0,
        }
    }
}

impl MeshEmitter<'_> {
    pub(super) fn emit(&mut self, list: &DisplayList) -> Result<(), &'static str> {
        let mut transforms = Vec::new();
        let mut clips = Vec::new();
        let mut depth = 0;
        let mut groups = 0;
        let commands = list.commands();
        let mut single_fill_opacity = None;
        for (index, command) in commands.iter().enumerate() {
            let (transform, slot) = transforms
                .last()
                .copied()
                .unwrap_or((AffineTransform::IDENTITY, 0));
            self.transform_slot = slot;
            self.gradient = 0;
            match command {
                PaintCommand::PushTransform(next) => {
                    let transform = transform.multiply(*next);
                    let slot = if self.mesh.retained_parameters {
                        self.mesh.push_parameters(transform, 1.0)
                    } else {
                        0
                    };
                    transforms.push((transform, slot));
                }
                PaintCommand::PopTransform => {
                    transforms.pop();
                }
                PaintCommand::PushClip(clip) => {
                    if let Some(mask) = ImageMask::from_clip(clip, transform, slot) {
                        let group = self.open_mask(mask, &mut groups);
                        clips.push((None, Some(group)));
                    } else {
                        let start = self.mesh.vertices.len();
                        clips.push((Some(self.clip(clip, transform, &mut depth)?), None));
                        self.mesh.rect_vertices[index] = start..self.mesh.vertices.len();
                    }
                }
                PaintCommand::PushLayer(layer) => {
                    let (transform, slot) = if layer.transform() == AffineTransform::IDENTITY {
                        (transform, slot)
                    } else {
                        let transform = transform.multiply(layer.transform());
                        let slot = if self.mesh.retained_parameters {
                            self.mesh.push_parameters(transform, 1.0)
                        } else {
                            0
                        };
                        (transform, slot)
                    };
                    transforms.push((transform, slot));
                    self.transform_slot = slot;
                    let blend_mode = layer.blend_mode();
                    let composition = layer.composition();
                    let blend = (blend_mode != BlendMode::Normal
                        || composition != CompositionMode::SrcOver)
                        .then(|| ((mix(blend_mode) as u32) << 8) | compose(composition) as u32);
                    let replaces_destination =
                        matches!(composition, CompositionMode::Src | CompositionMode::Clear);
                    let (color_filter, blur_radius) =
                        self.layer_filters(layer.effects(), layer.opacity(), transform)?;
                    let mask = layer
                        .clip()
                        .and_then(|clip| ImageMask::from_clip(clip, transform, slot));
                    if mask.is_some() && replaces_destination {
                        return Err("image mask with source replacement or clear");
                    }
                    let clip_start = self.mesh.vertices.len();
                    let clip = layer
                        .clip()
                        .filter(|_| mask.is_none())
                        .map(|clip| self.clip(clip, transform, &mut depth))
                        .transpose()?;
                    let replacement_bounds = replaces_destination.then(|| {
                        if clip.is_some() {
                            pixel_bounds(
                                &self.mesh.vertices[clip_start..],
                                &self.mesh.parameters,
                                self.extent,
                            )
                        } else {
                            [0, 0, self.extent.0, self.extent.1]
                        }
                    });
                    if !layer.opacity().is_finite() || !(0.0..=1.0).contains(&layer.opacity()) {
                        return Err("invalid layer opacity");
                    }
                    // A tessellated fill shades each covered sample once, so an
                    // isolated group containing just that fill can scale its alpha
                    // without allocating a target or changing how overlaps blend.
                    let single_fill = !layer.is_isolated()
                        && mask.is_none()
                        && blend.is_none()
                        && color_filter.is_none()
                        && blur_radius == 0.0
                        && (self.mesh.retained_parameters || layer.opacity() != 1.0)
                        && contains_one_fill(&commands[index + 1..]);
                    if single_fill {
                        single_fill_opacity = Some(layer.opacity());
                    }
                    let group = (layer.is_isolated()
                        || mask.is_some()
                        || blend.is_some()
                        || blur_radius > 0.0
                        || color_filter.is_some()
                        || (layer.opacity() != 1.0 && !single_fill))
                        .then(|| {
                            self.mesh.needs_blend |= blend.is_some();
                            self.mesh.group_marker(Operation::BeginGroup);
                            groups += 1;
                            self.mesh.group_depth = self.mesh.group_depth.max(groups);
                            PaintGroup {
                                mask,
                                blend,
                                replacement_bounds,
                                opacity: layer.opacity(),
                                vertex_start: self.mesh.vertices.len(),
                                color_filter,
                                blur_radius,
                                preceding_blurs: self.mesh.blur_count,
                            }
                        });
                    clips.push((clip, group));
                }
                PaintCommand::PopClip | PaintCommand::PopLayer => {
                    if matches!(command, PaintCommand::PopLayer) {
                        transforms.pop();
                    }
                    if let Some((clip, group)) = clips.pop() {
                        self.close_layer(clip, group, &mut depth, &mut groups)?;
                    }
                }
                PaintCommand::DrawPath { path, paint } => {
                    let stroke = match paint {
                        Paint::Fill(_) => None,
                        Paint::Stroke(stroke) => Some(StrokeGeometry::from_stroke(stroke)?),
                    };
                    let mut color = self.brush(paint.brush())?;
                    if let Some(opacity) = single_fill_opacity.take() {
                        if self.mesh.retained_parameters {
                            self.transform_slot = self.mesh.push_parameters(transform, opacity);
                        } else {
                            color.a = color.a.clamp(0.0, 1.0) * opacity;
                        }
                    }
                    let start = self.mesh.vertices.len();
                    let indices =
                        self.shape(Shape::Path(path, stroke.as_ref()), color, transform)?;
                    self.mesh.rect_vertices[index] = start..self.mesh.vertices.len();
                    self.mesh.draw(Operation::Paint, depth, indices);
                }
                PaintCommand::DrawText { origin, run, brush } => {
                    if run.is_empty() {
                        continue;
                    }
                    let color = self.brush(brush)?;
                    let start = self.mesh.vertices.len();
                    for glyph in run.glyphs() {
                        let shape = Shape::Glyph {
                            font: run.font(),
                            glyph: glyph.glyph_id(),
                            size: run.font_size(),
                            oblique: run.synth_oblique(),
                            bold: run.synth_bold(),
                        };
                        let offset = Point::new(origin.x + glyph.x(), origin.y + glyph.y());
                        self.draw_glyph(shape, color, transform, offset, depth, groups)?;
                    }
                    if let Some(vertices) = self.mesh.text_vertices.get_mut(index) {
                        *vertices = start..self.mesh.vertices.len();
                    }
                }
                PaintCommand::DrawShadow {
                    rect,
                    radii,
                    color,
                    blur,
                    spread,
                    offset,
                    inset,
                } => {
                    if *inset {
                        self.inset_shadow(
                            *rect, *radii, *color, *blur, *spread, *offset, transform, &mut depth,
                        )?;
                        continue;
                    }
                    let bounds = Rect::new(
                        Point::new(
                            rect.origin.x + offset.0 - spread,
                            rect.origin.y + offset.1 - spread,
                        ),
                        Size::new(
                            rect.size.width + 2.0 * spread,
                            rect.size.height + 2.0 * spread,
                        ),
                    );
                    if bounds.size.width <= 0.0 || bounds.size.height <= 0.0 {
                        continue;
                    }
                    let radii = radii.spread_offset(*spread);
                    let indices = if *blur > 0.0 {
                        let sigma = blur / 2.0;
                        let pad = 3.5 * sigma;
                        let quad = Rect::new(
                            Point::new(bounds.origin.x - pad, bounds.origin.y - pad),
                            Size::new(
                                bounds.size.width + 2.0 * pad,
                                bounds.size.height + 2.0 * pad,
                            ),
                        );
                        self.shadow_quad(
                            quad,
                            bounds,
                            *color,
                            shadow_radius(radii),
                            sigma,
                            transform,
                        )?
                    } else {
                        let shape = Path::rounded_rect(bounds, radii);
                        self.shape(Shape::Path(&shape, None), *color, transform)?
                    };
                    self.mesh.draw(Operation::Paint, depth, indices);
                }
                PaintCommand::DrawImage {
                    image,
                    tile,
                    region,
                    repeat_x,
                    repeat_y,
                } => {
                    if let Some((image, indices)) = self.image(
                        image,
                        *tile,
                        *region,
                        [*repeat_x, *repeat_y],
                        transform,
                        0.0,
                    )? {
                        self.mesh.draw(Operation::Image(image), depth, indices);
                    }
                }
                PaintCommand::BackdropFilter {
                    bounds,
                    radii,
                    blur,
                } => {
                    let radius = device_blur_radius(*blur, transform)?;
                    if radius > 0.0 && bounds.size.width > 0.0 && bounds.size.height > 0.0 {
                        let start = self.mesh.vertices.len();
                        let clip = Clip::rounded_rect(*bounds, *radii);
                        let indices = self.clip(&clip, transform, &mut depth)?;
                        let pixels = pixel_bounds(
                            &self.mesh.vertices[start..],
                            &self.mesh.parameters,
                            self.extent,
                        );
                        if pixels[2] > 0 && pixels[3] > 0 {
                            self.mesh.needs_backdrop = true;
                            self.mesh.group_marker(Operation::Backdrop {
                                sample_bounds: padded_bounds(pixels, radius, self.extent),
                                radius,
                                stencil_depth: depth,
                            });
                        }
                        self.mesh.draw(Operation::Pop, depth, indices);
                        depth -= 1;
                    }
                }
            }
        }
        while let Some((clip, group)) = clips.pop() {
            self.close_layer(clip, group, &mut depth, &mut groups)?;
        }
        Ok(())
    }

    fn image(
        &mut self,
        image: &SceneImage,
        tile: Rect,
        region: Rect,
        [repeat_x, repeat_y]: [bool; 2],
        transform: AffineTransform,
        shear: f32,
    ) -> Result<Option<(usize, Range<u32>)>, &'static str> {
        if image.width() == 0
            || image.height() == 0
            || tile.size.width <= 0.0
            || tile.size.height <= 0.0
            || region.size.width <= 0.0
            || region.size.height <= 0.0
        {
            return Ok(None);
        }
        if ![
            tile.origin.x,
            tile.origin.y,
            tile.right(),
            tile.bottom(),
            region.origin.x,
            region.origin.y,
            region.right(),
            region.bottom(),
        ]
        .into_iter()
        .all(f32::is_finite)
        {
            return Err("non-finite image geometry");
        }
        let repeat_x = repeat_x && tile.size.width > 0.5;
        let repeat_y = repeat_y && tile.size.height > 0.5;
        let axis = |repeat, low: f32, high: f32, tile_low, tile_high| {
            if repeat {
                (low, high)
            } else {
                (low.max(tile_low), high.min(tile_high))
            }
        };
        let (left, right) = axis(
            repeat_x,
            region.origin.x,
            region.right(),
            tile.origin.x,
            tile.right(),
        );
        let (top, bottom) = axis(
            repeat_y,
            region.origin.y,
            region.bottom(),
            tile.origin.y,
            tile.bottom(),
        );
        if right <= left || bottom <= top {
            return Ok(None);
        }
        let base = u32::try_from(self.mesh.vertices.len()).map_err(|_| "too many vertices")?;
        base.checked_add(3).ok_or("too many vertices")?;
        let start = u32::try_from(self.mesh.indices.len()).map_err(|_| "too many indices")?;
        let end = start.checked_add(6).ok_or("too many indices")?;
        for (x, y) in [(left, top), (right, top), (right, bottom), (left, bottom)] {
            let u = (x - tile.origin.x) / tile.size.width;
            let v = (y - tile.origin.y) / tile.size.height;
            if !u.is_finite() || !v.is_finite() {
                return Err("non-finite image coordinates");
            }
            self.mesh.vertices.push(Vertex {
                position: vertex_position(
                    self.mesh.retained_parameters,
                    transform,
                    Point::new(shear.mul_add(y - tile.origin.y, x), y),
                )?,
                color: [1.0; 4],
                shadow: [u, v, 0.0, 0.0],
                shadow_parameters: [f32::from(u8::from(repeat_x)), f32::from(u8::from(repeat_y))],
                transform: self.transform_slot,
                gradient: 0,
            });
        }
        self.mesh
            .indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        let images = &mut self.mesh.images;
        let index = images
            .iter()
            .position(|previous| previous == image)
            .unwrap_or_else(|| {
                let index = images.len();
                images.push(image.clone());
                index
            });
        Ok(Some((index, start..end)))
    }

    fn layer_filters(
        &mut self,
        effects: &[LayerEffect],
        opacity: f32,
        transform: AffineTransform,
    ) -> Result<(Option<u32>, f32), &'static str> {
        let mut matrix = None;
        let mut blur = None;
        for effect in effects {
            match effect {
                LayerEffect::Blend(_) => {}
                LayerEffect::ColorMatrix(value)
                    if matrix.is_none()
                        && value[19] == 0.0
                        && value.iter().all(|value| value.is_finite()) =>
                {
                    matrix = Some(value);
                }
                LayerEffect::Blur { radius }
                    if blur.is_none() && radius.is_finite() && *radius >= 0.0 =>
                {
                    blur = Some(device_blur_radius(*radius, transform)?);
                }
                _ => return Err("unsupported Lyon layer filter"),
            }
        }
        let radius = blur.unwrap_or(0.0);
        let Some(matrix) = matrix else {
            return Ok((None, radius));
        };
        let mut params = [0.0; 24];
        params[..20].copy_from_slice(matrix);
        params[20] = opacity;
        let filters = &mut self.mesh.color_filters;
        if let Some(slot) = filters.iter().position(|known| *known == params) {
            return Ok((
                Some(u32::try_from(slot).expect("bounded color filters")),
                radius,
            ));
        }
        if filters.len() >= MAX_COLOR_FILTERS {
            return Err("Lyon color-filter buffer is full");
        }
        let slot = u32::try_from(filters.len()).expect("bounded color filters");
        filters.push(params);
        Ok((Some(slot), radius))
    }

    fn close_layer(
        &mut self,
        clip: Option<Range<u32>>,
        group: Option<PaintGroup>,
        depth: &mut u32,
        groups: &mut usize,
    ) -> Result<(), &'static str> {
        // Copy and clear affect transparent source pixels too. Keep the layer
        // clip active for those operators. Other operators allow a blur halo.
        let keep_clip = group
            .as_ref()
            .is_some_and(|group| group.replacement_bounds.is_some());
        let mut clip = clip;
        if !keep_clip {
            self.pop_clip(clip.take(), depth);
        }
        if let Some(group) = group {
            let mut bounds = group.replacement_bounds.unwrap_or_else(|| {
                if group.preceding_blurs == self.mesh.blur_count {
                    pixel_bounds(
                        &self.mesh.vertices[group.vertex_start..],
                        &self.mesh.parameters,
                        self.extent,
                    )
                } else {
                    // Nested blur can produce pixels outside the underlying geometry.
                    [0, 0, self.extent.0, self.extent.1]
                }
            });
            if let Some(mask) = group.mask {
                self.apply_mask(mask, bounds, *depth, *groups)?;
            }
            if group.blur_radius > 0.0 {
                self.mesh.blur_count += 1;
                bounds = padded_bounds(bounds, group.blur_radius, self.extent);
            }
            self.mesh.group_marker(Operation::EndGroup {
                encode_srgb: false,
                blend: group.blend,
                opacity: group.opacity,
                bounds,
                color_filter: group.color_filter,
                blur_radius: group.blur_radius,
                stencil_depth: *depth,
            });
            *groups -= 1;
        }
        self.pop_clip(clip, depth);
        Ok(())
    }

    fn pop_clip(&mut self, clip: Option<Range<u32>>, depth: &mut u32) {
        if let Some(indices) = clip {
            self.mesh.draw(Operation::Pop, *depth, indices);
            *depth -= 1;
        }
    }

    fn brush(&mut self, brush: &Brush) -> Result<Color, &'static str> {
        if let Brush::Solid(solid) = brush {
            return Ok(solid.color());
        }
        if !self.mesh.retained_parameters {
            return Err("gradient requires local vertex coordinates");
        }
        self.gradient = self.mesh.gradients.insert(brush)?;
        Ok(Color::WHITE)
    }

    fn clip(
        &mut self,
        clip: &Clip,
        transform: AffineTransform,
        depth: &mut u32,
    ) -> Result<Range<u32>, &'static str> {
        if *depth >= 255 {
            return Err("stencil nesting deeper than 255 levels");
        }
        let path = match clip {
            Clip::Shape(ClipShape::Rect(rect)) => Path::rect(*rect),
            Clip::Shape(ClipShape::RoundedRect { rect, radii }) => {
                Path::rounded_rect(*rect, *radii)
            }
            Clip::Shape(ClipShape::Path(path)) => path.clone(),
            Clip::Mask(_) | Clip::ImageMask { .. } => return Err("mask clips"),
        };
        let indices = self.shape(Shape::Path(&path, None), Color::WHITE, transform)?;
        self.mesh.draw(Operation::Push, *depth, indices.clone());
        *depth += 1;
        Ok(indices)
    }

    #[allow(clippy::too_many_arguments)]
    fn inset_shadow(
        &mut self,
        outer: Rect,
        radii: CornerRadii,
        color: Color,
        blur: f32,
        spread: f32,
        offset: (f32, f32),
        transform: AffineTransform,
        depth: &mut u32,
    ) -> Result<(), &'static str> {
        if outer.size.width <= 0.0 || outer.size.height <= 0.0 {
            return Ok(());
        }
        let inner = Rect::new(
            Point::new(
                outer.origin.x + offset.0 + spread,
                outer.origin.y + offset.1 + spread,
            ),
            Size::new(
                outer.size.width - 2.0 * spread,
                outer.size.height - 2.0 * spread,
            ),
        );
        if inner.size.width <= 0.0 || inner.size.height <= 0.0 {
            let path = Path::rounded_rect(outer, radii);
            let indices = self.shape(Shape::Path(&path, None), color, transform)?;
            self.mesh.draw(Operation::Paint, *depth, indices);
            return Ok(());
        }

        // The geometry clip keeps even a displaced or expanded hole from
        // producing pixels outside the original rounded box.
        let clip = self.clip(&Clip::rounded_rect(outer, radii), transform, depth)?;
        let inner_radii = radii.spread_offset(-spread);
        let sigma = blur / 2.0;
        let indices = if sigma > 0.0 {
            self.shadow_quad(
                outer,
                inner,
                color,
                shadow_radius(inner_radii),
                -sigma,
                transform,
            )?
        } else {
            let ring = Path::rounded_rect_ring(outer, radii, inner, inner_radii);
            self.shape(Shape::Path(&ring, None), color, transform)?
        };
        self.mesh.draw(Operation::Paint, *depth, indices);
        self.mesh.draw(Operation::Pop, *depth, clip);
        *depth -= 1;
        Ok(())
    }

    fn shadow_quad(
        &mut self,
        quad: Rect,
        shape: Rect,
        color: Color,
        radius: f32,
        sigma: f32,
        transform: AffineTransform,
    ) -> Result<Range<u32>, &'static str> {
        let half = [shape.size.width / 2.0, shape.size.height / 2.0];
        let center = Point::new(shape.origin.x + half[0], shape.origin.y + half[1]);
        let radius = radius.min(half[0]).min(half[1]);
        let color = premultiply(color);
        let base = u32::try_from(self.mesh.vertices.len()).map_err(|_| "too many vertices")?;
        let start = u32::try_from(self.mesh.indices.len()).map_err(|_| "too many indices")?;
        let end = start.checked_add(6).ok_or("too many indices")?;
        base.checked_add(3).ok_or("too many vertices")?;
        let (left, top, right, bottom) =
            (quad.origin.x, quad.origin.y, quad.right(), quad.bottom());
        for (x, y) in [(left, top), (right, top), (right, bottom), (left, bottom)] {
            self.mesh.vertices.push(Vertex {
                position: vertex_position(
                    self.mesh.retained_parameters,
                    transform,
                    Point::new(x, y),
                )?,
                color,
                shadow: [x - center.x, y - center.y, half[0], half[1]],
                shadow_parameters: [radius, sigma],
                transform: self.transform_slot,
                gradient: self.gradient,
            });
        }
        self.mesh
            .indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        Ok(start..end)
    }
}

fn shadow_radius(radii: CornerRadii) -> f32 {
    radii
        .corners()
        .iter()
        .map(|corner| corner.circular_approximation())
        .fold(0.0, f32::max)
}
