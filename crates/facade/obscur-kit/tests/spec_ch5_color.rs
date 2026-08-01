#![doc = include_str!("spec_ch5_color.doc/_module.md")]
#![forbid(unsafe_code)]
#![cfg(feature = "render-cpu")]

#[path = "spec_ch5_support/mod.rs"]
mod support;

use support::{drive, el, rasterize, view};

#[doc = include_str!("spec_ch5_color.doc/VW.md")]
const VW: f32 = 200.0;

#[doc = include_str!("spec_ch5_color.doc/sc_03_opacity_composites_in_linear_light.md")]
#[test]
fn sc_03_opacity_composites_in_linear_light() {
    let rt = drive(
        VW,
        VW,
        ".back{position:absolute;left:0;top:0;width:100px;height:100px;\
               background-color:rgb(255,255,255)}\
         .front{position:absolute;left:0;top:0;width:100px;height:100px;\
                background-color:rgb(0,0,0);opacity:0.5}",
        &view(vec![el(10, "back", vec![]), el(20, "front", vec![])]),
    );

    let (r, g, b, a) = rasterize(&rt, VW, VW).at(50, 50);
    assert_eq!(a, 255, "the stack is opaque");
    assert_eq!(r, g, "the result stays neutral grey");
    assert_eq!(g, b, "the result stays neutral grey");
    assert!(
        (1..=254).contains(&r),
        "black at opacity 0.5 over white lands strictly between the two layers: \
         255 means `opacity` was ignored, 0 means the front was drawn opaque; \
         got {r}"
    );
}

#[doc = include_str!("spec_ch5_color.doc/sc_04_gradient_stops_interpolate_premultiplied.md")]
#[test]
fn sc_04_gradient_stops_interpolate_premultiplied() {
    let rt = drive(
        VW,
        VW,
        ".page{position:absolute;left:0;top:0;width:200px;height:200px;\
               background-color:rgb(255,255,255)}\
         .a{position:absolute;left:0;top:0;width:100px;height:100px;\
            background-image:linear-gradient(to bottom, transparent, rgb(0,0,255))}",
        &view(vec![el(10, "page", vec![]), el(20, "a", vec![])]),
    );

    let (r, g, b, _) = rasterize(&rt, VW, VW).at(50, 50);
    assert_eq!(
        b, 255,
        "the premultiplied midpoint of transparent→blue is full-strength blue \
         at half alpha; a blue channel near 191 is the un-premultiplied lerp \
         toward black"
    );
    assert_eq!(
        r, g,
        "the midpoint carries no red/green tint of its own (R={r}, G={g})"
    );
    // CORRECTED 2026-07-20 by the owner: this asserted (188,188,255) — linear-light
    // compositing — which contradicts the sc_03 ruling that plain source-over (a
    // gradient stop's alpha over its backdrop) composites in the element's own
    // GAMMA space, with linear reserved for blend modes, blur and filters. Half-
    // alpha blue over white in gamma is (1-a)·255 per channel not already
    // saturated. Pixel row 50's centre is at y=50.5, so the gradient fraction is
    // 0.505 and the blue alpha is 0.505 — not exactly 0.5; (1-0.505)·255 = 126.2,
    // giving R=G=126, B saturated at 255. The premultiplied-interpolation point
    // above (blue stays full-strength, not lerped toward black) is unchanged and is
    // the actual §5.16 requirement this case was written for.
    assert_eq!(
        (r, g, b),
        (126, 126, 255),
        "half-alpha blue over white, source-over in the element's gamma space"
    );
}
