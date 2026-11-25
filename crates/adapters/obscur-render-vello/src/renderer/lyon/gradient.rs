//! Encode gradient geometry and premultiplied stops for the fragment shader.

mod font_gradient_record;

use font_gradient_record::FontGradientRecord;
use obscur_scene::color::Color;
use obscur_scene::color::convert::srgb_decode;
use obscur_scene::paint::{Brush, ExtendMode, GradientStop};

use obscur_scene::color::convert::premul_oklab;

use super::font_paint::FontPaint;

// Match the uniform declaration in vector.wgsl. 1024 vec4s fit the 16 KiB
// minimum uniform binding size.
pub(super) const GRADIENT_ROWS: usize = 1024;

#[derive(Default)]
pub(super) struct Gradients {
    pub(super) rows: Vec<[f32; 4]>,
    brushes: Vec<(Brush, u32, usize)>,
    fonts: Vec<FontGradientRecord>,
}

impl Gradients {
    pub(super) fn encoded(&self, slot: usize) -> Option<&[[f32; 4]]> {
        let header = self.rows.get(slot)?;
        let geometry = if header[3] == 0.0 { 1 } else { 4 };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let stops = header[2] as usize;
        self.rows.get(slot..slot + 1 + geometry + 2 * stops)
    }

    pub(super) fn clear(&mut self) {
        self.rows.clear();
        self.brushes.clear();
        self.fonts.clear();
    }

    pub(super) fn insert_font(
        &mut self,
        paint: &FontPaint,
        foreground: Color,
    ) -> Result<u32, &'static str> {
        let stops = paint.stops(foreground);
        if let Some(record) = self.fonts.iter().find(|record| {
            record.kind == paint.kind
                && record.extend == paint.extend
                && record.geometry == paint.geometry
                && record.stops == stops
        }) {
            return Ok(record.index);
        }
        let index = self.encode(paint.kind, paint.extend, &paint.geometry, &stops, true)?;
        self.fonts.push(FontGradientRecord {
            kind: paint.kind,
            extend: paint.extend,
            geometry: paint.geometry,
            stops,
            index,
        });
        Ok(index)
    }

    pub(super) fn first_color(&self, index: u32) -> Option<Color> {
        let (brush, _, _) = self.brushes.iter().find(|(_, slot, _)| *slot == index)?;
        let stops = match brush {
            Brush::Linear(gradient) => gradient.stops(),
            Brush::Radial(gradient) => gradient.stops(),
            Brush::Conic(gradient) => gradient.stops(),
            Brush::Solid(_) => return None,
        };
        stops.first().map(|stop| stop.color())
    }

    pub(super) fn insert(&mut self, brush: &Brush) -> Result<u32, &'static str> {
        if let Some((_, index, uses)) = self.brushes.iter_mut().find(|(known, _, _)| known == brush)
        {
            *uses += 1;
            return Ok(*index);
        }
        let (kind, extend, geometry, stops) = match brush {
            Brush::Linear(g) => {
                let (start, end) = (g.start(), g.end());
                let dx = end.x - start.x;
                let dy = end.y - start.y;
                let length_squared = dx * dx + dy * dy;
                if length_squared <= 0.0 || !length_squared.is_finite() {
                    return Err("degenerate linear gradient");
                }
                (
                    0.0,
                    g.extend(),
                    [start.x, start.y, dx / length_squared, dy / length_squared],
                    g.stops(),
                )
            }
            Brush::Radial(g) => {
                if g.radius() <= 0.0 || g.radius_y() <= 0.0 {
                    return Err("degenerate radial gradient");
                }
                (
                    1.0,
                    g.extend(),
                    [
                        g.center().x,
                        g.center().y,
                        1.0 / g.radius(),
                        1.0 / g.radius_y(),
                    ],
                    g.stops(),
                )
            }
            Brush::Conic(g) => (
                2.0,
                g.extend(),
                [g.center().x, g.center().y, g.start_angle(), 0.0],
                g.stops(),
            ),
            Brush::Solid(_) => return Ok(0),
        };
        let index = self.encode(kind, extend, &[geometry], stops, false)?;
        self.brushes.push((brush.clone(), index, 1));
        Ok(index)
    }

    pub(super) fn update_linear(&mut self, index: u32, next: &Brush) -> bool {
        let Brush::Linear(next_gradient) = next else {
            return false;
        };
        let Some((brush, _, 1)) = self.brushes.iter_mut().find(|(_, slot, _)| *slot == index)
        else {
            return false;
        };
        let Brush::Linear(previous) = brush else {
            return false;
        };
        if previous.extend() != next_gradient.extend() || previous.stops() != next_gradient.stops()
        {
            return false;
        }
        let (start, end) = (next_gradient.start(), next_gradient.end());
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let length_squared = dx * dx + dy * dy;
        if length_squared <= 0.0 || !length_squared.is_finite() {
            return false;
        }
        self.rows[index as usize + 1] =
            [start.x, start.y, dx / length_squared, dy / length_squared];
        *brush = next.clone();
        true
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn encode(
        &mut self,
        kind: f32,
        extend: ExtendMode,
        geometry: &[[f32; 4]],
        stops: &[GradientStop],
        font: bool,
    ) -> Result<u32, &'static str> {
        if !geometry.iter().flatten().all(|value| value.is_finite())
            || stops.is_empty()
            || stops
                .iter()
                .any(|stop| !(0.0..=1.0).contains(&stop.offset()))
            || stops
                .windows(2)
                .any(|pair| pair[0].offset() > pair[1].offset())
        {
            return Err("invalid gradient geometry or stops");
        }
        let start = self.rows.len().max(1);
        if stops.len() > (GRADIENT_ROWS.saturating_sub(start + 1 + geometry.len())) / 2 {
            return Err("gradient stop buffer is full");
        }
        if self.rows.is_empty() {
            self.rows.push([0.0; 4]); // Vertex index zero denotes solid paint.
        }
        let extend = match extend {
            ExtendMode::Pad => 0.0,
            ExtendMode::Repeat => 1.0,
            ExtendMode::Reflect => 2.0,
        };
        self.rows
            .push([kind, extend, stops.len() as f32, f32::from(u8::from(font))]);
        self.rows.extend_from_slice(geometry);
        for stop in stops {
            self.rows.push([stop.offset(), 0.0, 0.0, 0.0]);
            let color = stop.color();
            self.rows.push(if font {
                let linear = |channel| srgb_decode(f64::from(channel)) as f32 * color.a;
                [linear(color.r), linear(color.g), linear(color.b), color.a]
            } else {
                premul_oklab(color).map(|value| value as f32)
            });
        }
        let index = u32::try_from(start).expect("bounded gradient buffer");
        Ok(index)
    }
}
