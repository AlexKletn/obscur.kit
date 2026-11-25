#![doc = include_str!("../../docs/api/renderer/group_compositing.doc/_module.md")]

use super::gaussian_blur::blur_layer;

#[doc = include_str!("../../docs/api/renderer/group_compositing.doc/composite_top_layer.md")]
pub(crate) fn composite_top_layer(
    targets: &mut Vec<tiny_skia::Pixmap>,
    opacity: f32,
    blend: tiny_skia::BlendMode,
    color_matrix: Option<&[f32; 20]>,
    blur: f32,
    enclosing_clip: Option<&tiny_skia::Mask>,
) {
    // Only composite an actual layer: never pop the root frame buffer.
    if targets.len() < 2 {
        return;
    }
    let Some(mut child) = targets.pop() else {
        return;
    };
    let Some(parent) = targets.last_mut() else {
        // Cannot happen (len was >= 2), but keep the buffer to avoid data loss.
        targets.push(child);
        return;
    };
    if let Some(matrix) = color_matrix {
        apply_color_matrix(&mut child, matrix);
    }
    if blur > 0.0 {
        blur_layer(&mut child, blur);
    }
    if matches!(
        blend,
        tiny_skia::BlendMode::Source | tiny_skia::BlendMode::Clear
    ) {
        composite_replacement(parent, &child, opacity, blend, enclosing_clip);
        return;
    }
    if matches!(
        blend,
        tiny_skia::BlendMode::Hue
            | tiny_skia::BlendMode::Saturation
            | tiny_skia::BlendMode::Color
            | tiny_skia::BlendMode::Luminosity
    ) {
        composite_nonseparable(
            parent,
            &child,
            opacity.clamp(0.0, 1.0),
            blend,
            enclosing_clip,
        );
        return;
    }
    let paint = tiny_skia::PixmapPaint {
        opacity: opacity.clamp(0.0, 1.0),
        blend_mode: blend,
        ..Default::default()
    };
    parent.draw_pixmap(
        0,
        0,
        child.as_ref(),
        &paint,
        tiny_skia::Transform::identity(),
        enclosing_clip,
    );
}

fn composite_replacement(
    target: &mut tiny_skia::Pixmap,
    source: &tiny_skia::Pixmap,
    opacity: f32,
    blend: tiny_skia::BlendMode,
    clip: Option<&tiny_skia::Mask>,
) {
    let source_scale = if blend == tiny_skia::BlendMode::Clear {
        0.0
    } else {
        opacity.clamp(0.0, 1.0)
    };
    for (index, (destination, source)) in target
        .data_mut()
        .chunks_exact_mut(4)
        .zip(source.data().chunks_exact(4))
        .enumerate()
    {
        let coverage = f32::from(clip.map_or(255, |mask| mask.data()[index])) / 255.0;
        for (destination, &source) in destination.iter_mut().zip(source) {
            let original = f32::from(*destination) * (1.0 - coverage);
            *destination = (original + f32::from(source) * source_scale * coverage).round() as u8;
        }
    }
}

// tiny-skia 0.11 clips negative channels incorrectly in its nonseparable modes.
fn composite_nonseparable(
    target: &mut tiny_skia::Pixmap,
    source: &tiny_skia::Pixmap,
    opacity: f32,
    blend: tiny_skia::BlendMode,
    clip: Option<&tiny_skia::Mask>,
) {
    for (index, (backdrop, source)) in target
        .data_mut()
        .chunks_exact_mut(4)
        .zip(source.data().chunks_exact(4))
        .enumerate()
    {
        let coverage = clip.map_or(255, |mask| mask.data()[index]);
        let source_alpha = f32::from(source[3]) / 255.0 * opacity * (f32::from(coverage) / 255.0);
        if source_alpha == 0.0 {
            continue;
        }
        let backdrop_alpha = f32::from(backdrop[3]) / 255.0;
        let straight = |pixel: &[u8]| {
            let alpha = f32::from(pixel[3]).max(1.0);
            [pixel[0], pixel[1], pixel[2]].map(|value| (f32::from(value) / alpha).min(1.0))
        };
        let b = straight(backdrop);
        let s = straight(source);
        let mixed = match blend {
            tiny_skia::BlendMode::Hue => {
                set_luminosity(set_saturation(s, saturation(b)), luminosity(b))
            }
            tiny_skia::BlendMode::Saturation => {
                set_luminosity(set_saturation(b, saturation(s)), luminosity(b))
            }
            tiny_skia::BlendMode::Color => set_luminosity(s, luminosity(b)),
            tiny_skia::BlendMode::Luminosity => set_luminosity(b, luminosity(s)),
            _ => s,
        };
        let result_alpha = source_alpha + backdrop_alpha * (1.0 - source_alpha);
        for channel in 0..3 {
            let value = b[channel] * backdrop_alpha * (1.0 - source_alpha)
                + s[channel] * source_alpha * (1.0 - backdrop_alpha)
                + mixed[channel] * source_alpha * backdrop_alpha;
            backdrop[channel] = (value.clamp(0.0, result_alpha) * 255.0).round() as u8;
        }
        backdrop[3] = (result_alpha * 255.0).round() as u8;
    }
}

fn luminosity([r, g, b]: [f32; 3]) -> f32 {
    r * 0.30 + g * 0.59 + b * 0.11
}

fn saturation([r, g, b]: [f32; 3]) -> f32 {
    r.max(g).max(b) - r.min(g).min(b)
}

fn set_saturation(color: [f32; 3], value: f32) -> [f32; 3] {
    let spread = saturation(color);
    if spread == 0.0 {
        return [0.0; 3];
    }
    let [r, g, b] = color;
    color.map(|channel| (channel - r.min(g).min(b)) * value / spread)
}

fn set_luminosity(color: [f32; 3], value: f32) -> [f32; 3] {
    let delta = value - luminosity(color);
    let [r, g, b] = color.map(|channel| channel + delta);
    let low = r.min(g).min(b);
    let high = r.max(g).max(b);
    let mut adjusted = [r, g, b];
    if low < 0.0 {
        adjusted = adjusted.map(|channel| value + (channel - value) * value / (value - low));
    }
    if high > 1.0 {
        adjusted =
            adjusted.map(|channel| value + (channel - value) * (1.0 - value) / (high - value));
    }
    adjusted
}

/// Applies a color filter in straight RGBA and writes premultiplied pixels.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn apply_color_matrix(pixmap: &mut tiny_skia::Pixmap, matrix: &[f32; 20]) {
    for pixel in pixmap.data_mut().chunks_exact_mut(4) {
        let alpha = f32::from(pixel[3]);
        if alpha == 0.0 {
            continue;
        }
        let rgba = [
            f32::from(pixel[0]) / alpha,
            f32::from(pixel[1]) / alpha,
            f32::from(pixel[2]) / alpha,
            alpha / 255.0,
        ];
        let mut output = [0.0; 4];
        for (index, (channel, row)) in output
            .iter_mut()
            .zip(matrix[..16].chunks_exact(4))
            .enumerate()
        {
            let value = row[0] * rgba[0]
                + row[1] * rgba[1]
                + row[2] * rgba[2]
                + row[3] * rgba[3]
                + matrix[16 + index];
            *channel = value.clamp(0.0, 1.0);
        }
        let alpha = output[3] * 255.0;
        for (channel, value) in pixel[..3].iter_mut().zip(output[..3].iter()) {
            *channel = (value * alpha).round() as u8;
        }
        pixel[3] = alpha.round() as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 1x1 pixmap filled with the given straight-alpha RGBA8 color.
    fn pixel(r: u8, g: u8, b: u8, a: u8) -> tiny_skia::Pixmap {
        let mut pixmap = tiny_skia::Pixmap::new(1, 1).expect("1x1 pixmap allocates");
        pixmap.fill(tiny_skia::Color::from_rgba8(r, g, b, a));
        pixmap
    }

    #[test]
    fn a_stray_pop_never_consumes_the_root_frame_buffer() {
        let mut targets = vec![pixel(255, 0, 0, 255)];
        composite_top_layer(
            &mut targets,
            0.5,
            tiny_skia::BlendMode::SourceOver,
            None,
            0.0,
            None,
        );
        assert_eq!(targets.len(), 1, "the root buffer survives a stray pop");
        let root = targets.pop().expect("root remains");
        assert_eq!(root.pixel(0, 0).expect("pixel").red(), 255);
    }

    #[test]
    fn the_offscreen_folds_back_at_the_layers_opacity() {
        // A white root with an opaque black layer over it at half opacity blends
        // to mid grey — one group composite, and the layer target is dropped.
        let mut targets = vec![pixel(255, 255, 255, 255), pixel(0, 0, 0, 255)];
        composite_top_layer(
            &mut targets,
            0.5,
            tiny_skia::BlendMode::SourceOver,
            None,
            0.0,
            None,
        );
        assert_eq!(targets.len(), 1, "the offscreen is consumed");
        let root = &targets[0];
        let blended = root.pixel(0, 0).expect("pixel").red();
        assert!(
            (100..=155).contains(&blended),
            "half-opacity black over white reads mid grey, got {blended}"
        );
    }
}
