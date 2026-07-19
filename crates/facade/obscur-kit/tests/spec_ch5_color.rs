//! **SPECIFICATION-DERIVED** colour-pipeline cases for obscur.kit §5.7 ("Color").
//!
//! §5.7 states two properties with numeric consequences: *"linear compositing for
//! blending, blur and filters"* and *"premultiplied alpha"*. Every expected value
//! below was produced by an agent reading that section and css-color-4 §10.2 /
//! §12.3, with **no access to this repository's source**.
//!
//! Note for readers: §5.7's "linear compositing" **contradicts CSS**, where
//! `opacity` and simple alpha compositing are defined on gamma-encoded sRGB. The
//! deriver took the toolkit's own spec at face value. If the toolkit decides CSS
//! wins, [`sc_03_opacity_composites_in_linear_light`]'s expected value becomes
//! `128` and §5.7 must be amended — but the expected value here may not be
//! changed to match whatever the code does.
#![forbid(unsafe_code)]
#![cfg(feature = "render-cpu")]

#[path = "spec_ch5_support/mod.rs"]
mod support;

use support::{drive, el, rasterize, view};

/// The viewport every case in this file renders under.
const VW: f32 = 200.0;

/// §5.7 "- linear compositing for blending, blur and filters;". sRGB transfer
/// function from css-color-4 §10.2: "cl = sign * (Math.pow((abs + 0.055) / 1.055,
/// 2.4));" / "c is the gamma-encoded red, green or blue component. cl is the
/// corresponding linear-light component."
///
/// Composite in linear light: `0.5*1.0 + 0.5*0.0 = 0.5` linear; encoding 0.5 back
/// to sRGB gives `1.055*0.5^(1/2.4) - 0.055 = 0.7354` → **188**. An engine that
/// composites directly on gamma-encoded sRGB bytes gives 128, a 60-unit gap far
/// outside any anti-aliasing tolerance §5.2 permits.
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
    assert_eq!(
        (r, g, b),
        (188, 188, 188),
        "black at opacity 0.5 over white composites in LINEAR light to 188; \
         188 means linear, 128 means the composite ran on gamma-encoded bytes"
    );
}

/// §5.7 "- premultiplied alpha;", reinforced by css-color-4 §12.3: "Interpolating
/// colors using the premultiplied representations tends to produce more attractive
/// transitions than the non-premultiplied representations".
///
/// The premultiplied interpolation of `transparent` → blue yields
/// `rgba(0,0,255,0.5)` at the midpoint — **full-strength** blue at half alpha.
/// Un-premultiplied interpolation treats `transparent` as `rgba(0,0,0,0)` and
/// lerps the colour toward black, giving `rgb(0,0,128)` at alpha 0.5 — the classic
/// "fade-to-transparent goes grey" bug. The `B == 255` assertion isolates that and
/// is independent of whether compositing is linear or sRGB.
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
    // Under §5.7's linear compositing the composited value is rgb(188,188,255).
    assert_eq!(
        (r, g, b),
        (188, 188, 255),
        "half-alpha blue over white, composited in linear light"
    );
}
