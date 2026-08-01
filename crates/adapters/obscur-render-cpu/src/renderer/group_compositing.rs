#![doc = include_str!("group_compositing.doc/_module.md")]

#[doc = include_str!("group_compositing.doc/composite_top_layer.md")]
pub(crate) fn composite_top_layer(
    targets: &mut Vec<tiny_skia::Pixmap>,
    opacity: f32,
    blend: tiny_skia::BlendMode,
) {
    // Only composite an actual layer: never pop the root frame buffer.
    if targets.len() < 2 {
        return;
    }
    let Some(child) = targets.pop() else {
        return;
    };
    let Some(parent) = targets.last_mut() else {
        // Cannot happen (len was >= 2), but keep the buffer to avoid data loss.
        targets.push(child);
        return;
    };
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
        None,
    );
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
        composite_top_layer(&mut targets, 0.5, tiny_skia::BlendMode::SourceOver);
        assert_eq!(targets.len(), 1, "the root buffer survives a stray pop");
        let root = targets.pop().expect("root remains");
        assert_eq!(root.pixel(0, 0).expect("pixel").red(), 255);
    }

    #[test]
    fn the_offscreen_folds_back_at_the_layers_opacity() {
        // A white root with an opaque black layer over it at half opacity blends
        // to mid grey — one group composite, and the layer target is dropped.
        let mut targets = vec![pixel(255, 255, 255, 255), pixel(0, 0, 0, 255)];
        composite_top_layer(&mut targets, 0.5, tiny_skia::BlendMode::SourceOver);
        assert_eq!(targets.len(), 1, "the offscreen is consumed");
        let root = &targets[0];
        let blended = root.pixel(0, 0).expect("pixel").red();
        assert!(
            (100..=155).contains(&blended),
            "half-opacity black over white reads mid grey, got {blended}"
        );
    }
}
