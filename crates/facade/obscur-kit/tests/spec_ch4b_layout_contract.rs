#![doc = include_str!("spec_ch4b_layout_contract.doc/_module.md")]

mod spec_ch4b_support;

use obscur_kit::geometry::{BoxConstraints, Size};
use obscur_kit::identity::NodeId;
use obscur_kit::layout::algorithm::{ChildMeasurer, CustomLayout, LayoutAlgorithm, LayoutChild};
use obscur_kit::layout::model::LayoutStyle;

#[doc = include_str!("spec_ch4b_layout_contract.doc/PackingMeasurer.md")]
struct PackingMeasurer;

impl ChildMeasurer for PackingMeasurer {
    fn measure_child(&mut self, index: usize, constraints: BoxConstraints) -> Size {
        // A wrapping/packing shape: widths cycle so the layout has no single
        // natural measure order, heights vary with the index.
        #[allow(clippy::cast_precision_loss)]
        let w = 10.0 + ((index % 7) as f32) * 3.0;
        #[allow(clippy::cast_precision_loss)]
        let h = 5.0 + ((index % 11) as f32) * 2.0;
        constraints.constrain(Size::new(w, h))
    }
}

#[doc = include_str!("spec_ch4b_layout_contract.doc/serialize_rects.md")]
fn serialize_rects() -> String {
    let children: Vec<LayoutChild> = (0..200)
        .map(|i| LayoutChild::new(NodeId::from_raw(i + 1), LayoutStyle::default()))
        .collect();
    let layout = CustomLayout::new("packing");
    let out = layout.layout(
        &LayoutStyle::default(),
        &children,
        BoxConstraints::loose(Size::new(300.0, 300.0)),
        &mut PackingMeasurer,
    );
    let mut s = String::new();
    for b in out.child_boxes() {
        s.push_str(&format!(
            "{},{},{},{}\n",
            b.origin().x,
            b.origin().y,
            b.size().width,
            b.size().height
        ));
    }
    s
}

#[doc = include_str!("spec_ch4b_layout_contract.doc/ch4_2_c_custom_layout_is_deterministic_across_processes.md")]
#[test]
fn ch4_2_c_custom_layout_is_deterministic_across_processes() {
    // The child half of the test: a fresh process re-executes this same binary
    // with the marker set, prints the rects, and exits.
    if std::env::var_os("OBSCUR_CH4B_LAYOUT_CHILD").is_some() {
        print!("{}", serialize_rects());
        return;
    }

    let exe = std::env::current_exe().expect("the test binary's own path");
    let run = || {
        let out = std::process::Command::new(&exe)
            .args([
                "--exact",
                "ch4_2_c_custom_layout_is_deterministic_across_processes",
                "--nocapture",
            ])
            .env("OBSCUR_CH4B_LAYOUT_CHILD", "1")
            .output()
            .expect("the child test process ran");
        assert!(out.status.success(), "child process failed: {out:?}");
        String::from_utf8(out.stdout).expect("utf-8 child output")
    };

    let first = run();
    let second = run();
    assert!(
        first.contains(','),
        "the child process emitted no rects: {first:?}"
    );
    assert_eq!(
        first, second,
        "two separately-launched processes laid out the same 200-child custom \
         layout differently"
    );
}
