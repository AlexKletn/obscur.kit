//! §6.15 Insets and virtual keyboard — CSS environment values.
//!
//! Specification-derived. The expected numbers below come from the case's host
//! setup (a notched phone reporting a 44px top and a 34px bottom safe-area
//! inset), never from any constant in this repository.
//!
//! NOTE ON THE SETUP. §6.15 requires "Typed environment values: - safe areas"
//! *and* "CSS environment values may expose safe-area insets". This crate
//! exposes no API by which a host supplies safe-area insets to the runtime, so
//! the first half of the setup cannot be written; what remains writable is the
//! second half — whether `env(safe-area-inset-*)` is a resolvable value function
//! in the CSS pipeline at all. That is the load-bearing observable named in the
//! case, and it is asserted here.

#![forbid(unsafe_code)]

mod spec_ch6b_support;

use obscur_kit::style::computed::ComputedValue;
use obscur_kit::style::property::{Length, PropertyId};
use spec_ch6b_support::{at, drive, el, origin_of, view};

/// Host: top safe-area inset 44 CSS px, bottom safe-area inset 34 CSS px.
const CSS: &str = "#root { width: 400px; height: 800px; \
                   padding-top: env(safe-area-inset-top); \
                   padding-bottom: env(safe-area-inset-bottom) } \
                   #child { width: 10px; height: 10px }";

fn px_of(v: Option<&ComputedValue>) -> Option<f32> {
    match v?.as_length()? {
        Length::Px(p) => Some(p.0),
        _ => None,
    }
}

/// §6.15 "Typed environment values: - safe areas;" / "CSS environment values may
/// expose safe-area insets."
///
/// `#root { padding-top: env(safe-area-inset-top) }` under a host reporting a
/// 44px top inset yields a used `padding-top` of 44px, and the content box
/// starts at y=44. The declaration is not dropped as invalid.
#[test]
fn env_safe_area_inset_top_resolves_to_the_host_inset() {
    let v = view(vec![el(1, "root", vec![el(2, "child", vec![])])]);
    let rt = drive(400.0, 800.0, CSS, &v);

    let root = at(&rt, &[0]);
    let style = rt.computed_style(root).expect("a computed style for #root");

    assert_eq!(
        px_of(style.get(PropertyId::PaddingTop)),
        Some(44.0),
        "env(safe-area-inset-top) must resolve to the host's 44px top safe-area \
         inset; it computed to {:?} instead",
        style.get(PropertyId::PaddingTop),
    );

    let child = at(&rt, &[0, 0]);
    let (_, y) = origin_of(&rt, child);
    assert_eq!(
        y, 44.0,
        "#root's content box must start at y=44 under a 44px top safe-area inset"
    );
}

/// §6.15 "Typed environment values: - safe areas;" / "CSS environment values may
/// expose safe-area insets."
///
/// The bottom inset resolves independently of the top one: a host reporting a
/// 34px bottom inset gives `#root` a used `padding-bottom` of 34px.
#[test]
fn env_safe_area_inset_bottom_resolves_to_the_host_inset() {
    let v = view(vec![el(1, "root", vec![el(2, "child", vec![])])]);
    let rt = drive(400.0, 800.0, CSS, &v);

    let root = at(&rt, &[0]);
    let style = rt.computed_style(root).expect("a computed style for #root");

    assert_eq!(
        px_of(style.get(PropertyId::PaddingBottom)),
        Some(34.0),
        "env(safe-area-inset-bottom) must resolve to the host's 34px bottom \
         safe-area inset; it computed to {:?} instead",
        style.get(PropertyId::PaddingBottom),
    );
}
