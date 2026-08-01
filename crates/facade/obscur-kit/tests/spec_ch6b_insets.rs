#![doc = include_str!("spec_ch6b_insets.doc/_module.md")]
#![forbid(unsafe_code)]

mod spec_ch6b_support;

use obscur_kit::geometry::Insets;
use obscur_kit::style::computed::ComputedValue;
use obscur_kit::style::property::{Length, PropertyId};
use spec_ch6b_support::{at, drive_with_insets, el, origin_of, view};

#[doc = include_str!("spec_ch6b_insets.doc/HOST_INSETS.md")]
const HOST_INSETS: Insets = Insets {
    top: 44.0,
    right: 0.0,
    bottom: 34.0,
    left: 0.0,
};

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

#[doc = include_str!("spec_ch6b_insets.doc/env_safe_area_inset_top_resolves_to_the_host_inset.md")]
#[test]
fn env_safe_area_inset_top_resolves_to_the_host_inset() {
    let v = view(vec![el(1, "root", vec![el(2, "child", vec![])])]);
    let rt = drive_with_insets(400.0, 800.0, CSS, &v, HOST_INSETS);

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

#[doc = include_str!("spec_ch6b_insets.doc/env_safe_area_inset_bottom_resolves_to_the_host_inset.md")]
#[test]
fn env_safe_area_inset_bottom_resolves_to_the_host_inset() {
    let v = view(vec![el(1, "root", vec![el(2, "child", vec![])])]);
    let rt = drive_with_insets(400.0, 800.0, CSS, &v, HOST_INSETS);

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
