#![doc = include_str!("spec_ch5_images.doc/_module.md")]
#![forbid(unsafe_code)]
#![cfg(all(
    feature = "resource-fs",
    feature = "resource-decode",
    feature = "render-cpu"
))]

#[path = "spec_ch5_support/mod.rs"]
mod support;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::geometry::{Rect, Size};
use obscur_kit::identity::RuntimeId;
use obscur_kit::primitives::Image;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewProperty, ViewValue};

use support::{at, compile, geom};

#[doc = include_str!("spec_ch5_images.doc/VW.md")]
const VW: f32 = 400.0;

#[doc = include_str!("spec_ch5_images.doc/COUNTER.md")]
static COUNTER: AtomicU32 = AtomicU32::new(0);

// ---------------------------------------------------------------------------
// A minimal RGB8 PNG writer, so a 400×200 asset can exist without a dependency.
// ---------------------------------------------------------------------------

#[doc = include_str!("spec_ch5_images.doc/crc32.md")]
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[doc = include_str!("spec_ch5_images.doc/adler32.md")]
fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1_u32, 0_u32);
    for &byte in bytes {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

#[doc = include_str!("spec_ch5_images.doc/chunk.md")]
fn chunk(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 12);
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(payload);
    let mut crc_input = kind.to_vec();
    crc_input.extend_from_slice(payload);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
    out
}

#[doc = include_str!("spec_ch5_images.doc/solid_png.md")]
fn solid_png(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
    // Raw scanlines: one filter byte (0 = None) plus width RGB triples per row.
    let mut raw = Vec::with_capacity((height * (1 + width * 3)) as usize);
    for _ in 0..height {
        raw.push(0);
        for _ in 0..width {
            raw.extend_from_slice(&rgb);
        }
    }

    // zlib: header, stored deflate blocks, adler32.
    let mut z = vec![0x78, 0x01];
    let mut rest = raw.as_slice();
    while !rest.is_empty() {
        let take = rest.len().min(0xFFFF);
        let (block, tail) = rest.split_at(take);
        z.push(u8::from(tail.is_empty()));
        z.extend_from_slice(&(take as u16).to_le_bytes());
        z.extend_from_slice(&(!(take as u16)).to_le_bytes());
        z.extend_from_slice(block);
        rest = tail;
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit, truecolor RGB

    let mut png = vec![137, 80, 78, 71, 13, 10, 26, 10];
    png.extend_from_slice(&chunk(b"IHDR", &ihdr));
    png.extend_from_slice(&chunk(b"IDAT", &z));
    png.extend_from_slice(&chunk(b"IEND", &[]));
    png
}

#[doc = include_str!("spec_ch5_images.doc/asset_dir.md")]
fn asset_dir(assets: &[(&str, Vec<u8>)]) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("obscur-kit-ch5-img-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    for (name, bytes) in assets {
        std::fs::write(dir.join(name), bytes).expect("write asset");
    }
    dir
}

#[doc = include_str!("spec_ch5_images.doc/runtime_at.md")]
fn runtime_at(dir: &PathBuf) -> UiRuntime {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    KitBootstrap::new().install_image_resources(&mut runtime, dir);
    runtime
}

#[doc = include_str!("spec_ch5_images.doc/picture_view.md")]
fn picture_view() -> View {
    let picture = Image::new("pic.png")
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("pic".into())));
    let sibling = obscur_kit::view::view::ViewNode::element(
        obscur_kit::identity::ComponentId::from_raw(0xB0B),
    )
    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("sibling".into())));
    View::from_fragment(ViewFragment::from_nodes(vec![picture, sibling]))
}

#[doc = include_str!("spec_ch5_images.doc/drawn.md")]
fn drawn(runtime: &UiRuntime) -> (Rect, Rect) {
    let list = runtime.snapshot().display_list().commands().to_vec();
    let mut found = None;
    for command in &list {
        if let PaintCommand::DrawImage { tile, region, .. } = command {
            assert!(found.is_none(), "exactly one DrawImage per frame");
            found = Some((*tile, *region));
        }
    }
    found.unwrap_or_else(|| {
        panic!(
            "the replaced element paints its image; the display list is {:?}",
            list.iter().map(support::describe).collect::<Vec<_>>()
        )
    })
}

#[doc = include_str!("spec_ch5_images.doc/render.md")]
fn render(dir: &PathBuf, css: &str) -> UiRuntime {
    let sheet: Stylesheet = compile(css);
    let mut runtime = runtime_at(dir);
    let _ = runtime.render_now(&picture_view(), std::slice::from_ref(&sheet));
    let _ = runtime.poll_resource_tasks();
    let _ = runtime.render_now(&picture_view(), std::slice::from_ref(&sheet));
    runtime
}

#[doc = include_str!("spec_ch5_images.doc/quad.md")]
fn quad(rect: Rect) -> (f32, f32, f32, f32) {
    (
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
    )
}

#[doc = include_str!("spec_ch5_images.doc/im_01_object_fit_none_centres_at_natural_size_and_clips.md")]
#[test]
fn im_01_object_fit_none_centres_at_natural_size_and_clips() {
    let dir = asset_dir(&[("pic.png", solid_png(400, 200, [255, 0, 0]))]);
    let runtime = render(
        &dir,
        ".pic{position:absolute;left:0;top:0;width:100px;height:100px;object-fit:none}\
         .sibling{position:absolute;left:110px;top:0;width:50px;height:50px;\
                  background-color:rgb(0,0,255)}",
    );

    let (tile, region) = drawn(&runtime);
    assert_eq!(
        quad(tile),
        (-150.0, -50.0, 400.0, 200.0),
        "the concrete object size is the natural 400x200, centred in the \
         100x100 content box"
    );
    assert_eq!(
        quad(region),
        (0.0, 0.0, 100.0, 100.0),
        "the mandatory clip is the content box — zero pixels paint outside it"
    );

    let px = support::rasterize(&runtime, VW, VW);
    let (r, g, b, _) = px.at(130, 20);
    assert_eq!(
        (r, g, b),
        (0, 0, 255),
        "the sibling at (110,0) is unobscured; an unclipped 400x200 image would \
         paint over it"
    );
}

#[doc = include_str!("spec_ch5_images.doc/im_02_object_fit_contain_geometry_and_position.md")]
#[test]
fn im_02_object_fit_contain_geometry_and_position() {
    let dir = asset_dir(&[("pic.png", solid_png(400, 200, [255, 0, 0]))]);
    let base = ".pic{position:absolute;left:0;top:0;width:100px;height:100px;\
                object-fit:contain";

    for (position, expected) in [
        ("", (0.0, 25.0, 100.0, 50.0)),
        (";object-position:0% 0%", (0.0, 0.0, 100.0, 50.0)),
        (";object-position:100% 100%", (0.0, 50.0, 100.0, 50.0)),
    ] {
        let runtime = render(&dir, &format!("{base}{position}}}"));
        let (tile, _) = drawn(&runtime);
        assert_eq!(
            quad(tile),
            expected,
            "object-position{:?}: contain gives a 100x50 concrete size \
             (cover would give 200x100)",
            if position.is_empty() {
                " (initial 50% 50%)"
            } else {
                position
            }
        );
    }
}

#[doc = include_str!("spec_ch5_images.doc/im_03_a_2x_asset_has_a_halved_logical_intrinsic_size.md")]
#[test]
fn im_03_a_2x_asset_has_a_halved_logical_intrinsic_size() {
    let dir = asset_dir(&[("pic@2x.png", solid_png(400, 200, [255, 0, 0]))]);
    let sheet = compile(".pic{position:absolute;left:0;top:0;width:auto;height:auto}");
    let picture = Image::new("pic@2x.png")
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("pic".into())));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![picture]));

    for scale in [1.0_f32, 2.0] {
        let mut runtime = runtime_at(&dir);
        runtime.set_device_scale(scale);
        // No executor: the load is deferred out of the frame phase (§3.4/§5.16);
        // drive it, then re-render so the auto/auto box sizes from the decoded
        // intrinsic size.
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
        let _ = runtime.poll_resource_tasks();
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

        let (_, _, w, h) = geom(&runtime, at(&runtime, &[0]));
        assert_eq!(
            (w, h),
            (400.0, 200.0),
            "at window scale {scale} the auto/auto image takes its intrinsic \
             size in LOGICAL units, unchanged by the device scale"
        );
    }
}

#[test]
fn contained_image_uses_empty_natural_size_on_contained_axes() {
    let dir = asset_dir(&[("pic.png", solid_png(40, 20, [255, 0, 0]))]);
    let picture = Image::new("pic.png")
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("pic".into())));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![picture]));
    for (containment, expected) in [("size", (10.0, 10.0)), ("inline-size", (10.0, 20.0))] {
        let sheet = compile(&format!(
            ".pic {{ container-type: {containment}; box-sizing: border-box; padding: 5px; }}"
        ));
        let mut runtime = runtime_at(&dir);
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
        let _ = runtime.poll_resource_tasks();
        let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
        let (_, _, width, height) = geom(&runtime, at(&runtime, &[0]));
        assert_eq!((width, height), expected, "{containment}");
    }
}
