//! Separable Gaussian convolution over premultiplied linear light.

use obscur_scene::color::convert::{srgb_decode, srgb_encode};

/// Bounds the work per pixel on the software renderer.
pub(super) const MAX_SIGMA: f32 = 16.0;

pub(super) fn blur_layer(pixmap: &mut tiny_skia::Pixmap, sigma: f32) {
    let width = pixmap.width() as usize;
    let height = pixmap.height() as usize;
    let (mut left, mut top, mut right, mut bottom) = (width, height, 0, 0);
    for (y, row) in pixmap.data().chunks_exact(width * 4).enumerate() {
        let mut pixels = row.chunks_exact(4);
        if let Some(first) = pixels.position(|pixel| pixel[3] != 0) {
            let last = pixels
                .rposition(|pixel| pixel[3] != 0)
                .map_or(first, |index| first + 1 + index);
            left = left.min(first);
            right = right.max(last + 1);
            top = top.min(y);
            bottom = y + 1;
        }
    }
    if bottom == 0 {
        return;
    }

    let radius = kernel_radius(sigma);
    let x0 = left.saturating_sub(radius);
    let y0 = top.saturating_sub(radius);
    let x1 = (right + radius).min(width);
    let y1 = (bottom + radius).min(height);
    if (x0, y0, x1, y1) == (0, 0, width, height) {
        blur_region(pixmap, sigma);
        return;
    }
    let Some(region) = crop_rect(x0, y0, x1, y1) else {
        return;
    };
    let Some(mut crop) = pixmap.clone_rect(region) else {
        return;
    };
    blur_region(&mut crop, sigma);
    let stride = (x1 - x0) * 4;
    for (y, row) in crop.data().chunks_exact(stride).enumerate() {
        let start = ((y + y0) * width + x0) * 4;
        pixmap.data_mut()[start..start + stride].copy_from_slice(row);
    }
}

fn crop_rect(left: usize, top: usize, right: usize, bottom: usize) -> Option<tiny_skia::IntRect> {
    tiny_skia::IntRect::from_xywh(
        i32::try_from(left).ok()?,
        i32::try_from(top).ok()?,
        u32::try_from(right.checked_sub(left)?).ok()?,
        u32::try_from(bottom.checked_sub(top)?).ok()?,
    )
}

fn kernel_radius(sigma: f32) -> usize {
    (3.0 * sigma.min(MAX_SIGMA)).ceil() as usize
}

fn blur_region(pixmap: &mut tiny_skia::Pixmap, sigma: f32) {
    let sigma = sigma.min(MAX_SIGMA);
    let radius = kernel_radius(sigma);
    let mut weights: Vec<_> = (0..=2 * radius)
        .map(|tap| {
            let distance = tap.abs_diff(radius) as f32 / sigma;
            (-0.5 * distance * distance).exp()
        })
        .collect();
    let total: f32 = weights.iter().sum();
    for weight in &mut weights {
        *weight /= total;
    }

    let width = pixmap.width() as usize;
    let height = pixmap.height() as usize;
    let mut intermediate = vec![[0.0; 4]; width * height];
    let mut row = vec![[0.0; 4]; width];
    for (y, pixels) in pixmap.data().chunks_exact(width * 4).enumerate() {
        for (pixel, linear) in pixels.chunks_exact(4).zip(&mut row) {
            let alpha = f32::from(pixel[3]) / 255.0;
            *linear = [0.0, 0.0, 0.0, alpha];
            if alpha > 0.0 {
                for (channel, &encoded) in linear[..3].iter_mut().zip(pixel) {
                    let straight = (f64::from(encoded) / f64::from(pixel[3])).min(1.0);
                    *channel = srgb_decode(straight) as f32 * alpha;
                }
            }
        }
        for x in 0..width {
            intermediate[y * width + x] = convolve(x, width, radius, &weights, |x| row[x]);
        }
    }
    for (index, pixel) in pixmap.data_mut().chunks_exact_mut(4).enumerate() {
        let x = index % width;
        let y = index / width;
        let linear = convolve(y, height, radius, &weights, |y| intermediate[y * width + x]);
        let alpha = linear[3].clamp(0.0, 1.0);
        if alpha > 0.0 {
            for (channel, value) in pixel[..3].iter_mut().zip(linear) {
                let straight = (value / alpha).clamp(0.0, 1.0);
                *channel =
                    (srgb_encode(f64::from(straight)) * f64::from(alpha) * 255.0).round() as u8;
            }
        } else {
            pixel[..3].fill(0);
        }
        pixel[3] = (alpha * 255.0).round() as u8;
    }
}

/// Replaces the backdrop inside an antialiased mask with its blurred pixels.
pub(super) fn blur_backdrop(
    target: &mut tiny_skia::Pixmap,
    coverage: &tiny_skia::Mask,
    sigma: f32,
) {
    // Mask samples and target pixels must use the same row stride.
    if coverage.width() != target.width() || coverage.height() != target.height() {
        return;
    }
    let width = target.width() as usize;
    let height = target.height() as usize;
    let (mut left, mut top, mut right, mut bottom) = (width, height, 0, 0);
    for (y, row) in coverage.data().chunks_exact(width).enumerate() {
        if let Some(first) = row.iter().position(|&amount| amount != 0) {
            let last = row.iter().rposition(|&amount| amount != 0).unwrap_or(first);
            left = left.min(first);
            right = right.max(last + 1);
            top = top.min(y);
            bottom = y + 1;
        }
    }
    if bottom == 0 {
        return;
    }

    // Keep the kernel's sampling margin outside the mask.
    let margin = kernel_radius(sigma);
    let x0 = left.saturating_sub(margin);
    let y0 = top.saturating_sub(margin);
    let x1 = (right + margin).min(width);
    let y1 = (bottom + margin).min(height);
    let Some(region) = crop_rect(x0, y0, x1, y1) else {
        return;
    };
    let Some(mut filtered) = target.clone_rect(region) else {
        return;
    };
    blur_region(&mut filtered, sigma);

    let region_width = x1 - x0;
    for y in top..bottom {
        let start = y * width + left;
        let end = y * width + right;
        let filtered_start = (y - y0) * region_width + left - x0;
        let filtered_end = filtered_start + right - left;
        for ((pixel, blurred), &amount) in target.data_mut()[start * 4..end * 4]
            .chunks_exact_mut(4)
            .zip(filtered.data()[filtered_start * 4..filtered_end * 4].chunks_exact(4))
            .zip(&coverage.data()[start..end])
        {
            let amount = u16::from(amount);
            if amount > 0 {
                for (channel, &source) in pixel.iter_mut().zip(blurred) {
                    let mixed = u16::from(*channel) * (255 - amount) + u16::from(source) * amount;
                    *channel = ((mixed + 127) / 255) as u8;
                }
            }
        }
    }
}

fn convolve(
    position: usize,
    extent: usize,
    radius: usize,
    weights: &[f32],
    sample: impl Fn(usize) -> [f32; 4],
) -> [f32; 4] {
    let mut result = [0.0; 4];
    let start = position.saturating_sub(radius);
    let end = (position + radius + 1).min(extent);
    for index in start..end {
        let weight = weights[index + radius - position];
        for (channel, value) in result.iter_mut().zip(sample(index)) {
            *channel += value * weight;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mask_with_a_different_stride_leaves_the_target_intact() {
        let mut target = tiny_skia::Pixmap::new(3, 2).unwrap();
        target.data_mut()[..4].copy_from_slice(&[255, 0, 0, 255]);
        let before = target.clone();
        // Equal pixel counts would pass a buffer-length check, but the rows
        // describe a different surface.
        let mut mask = tiny_skia::Mask::new(2, 3).unwrap();
        mask.data_mut().fill(255);

        blur_backdrop(&mut target, &mask, 1.0);

        assert_eq!(target.data(), before.data());
    }

    #[test]
    fn cropping_to_coverage_and_kernel_margin_preserves_every_pixel() {
        let mut pixmap = tiny_skia::Pixmap::new(53, 37).unwrap();
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(24, 120, 230, 142);
        for (x, y) in [(15.0, 9.0), (0.0, 0.0), (47.0, 30.0)] {
            pixmap.data_mut().fill(0);
            pixmap.fill_rect(
                tiny_skia::Rect::from_xywh(x, y, 6.0, 7.0).unwrap(),
                &paint,
                tiny_skia::Transform::identity(),
                None,
            );
            let mut reference = pixmap.clone();
            blur_region(&mut reference, 2.3);
            blur_layer(&mut pixmap, 2.3);
            assert_eq!(pixmap.data(), reference.data(), "source at ({x}, {y})");
        }
    }
}
