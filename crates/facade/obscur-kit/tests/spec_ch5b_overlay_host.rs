//! §5.4 Overlay host mode — specification-derived conformance cases.
//!
//! Every expected value here was derived from the specification text quoted in
//! each test's doc comment, not from this repository's source. A failure is a
//! claim about the implementation.
#![forbid(unsafe_code)]

mod spec_ch5b_support;

use spec_ch5b_support::harness::{MockOverlayHost, approx, el, view};

/// A 100x50 filled rectangle, the whole mounted tree.
const RECT_CSS: &str = ".rect { width: 100px; height: 50px; background-color: rgb(255,0,0); }\n";

/// A 1000 ms linear opacity animation from 1.0 to 0.0.
const FADE_CSS: &str = "\
.rect { width: 100px; height: 50px; background-color: rgb(255,0,0); }\n\
@keyframes fade { from { opacity: 1.0; } to { opacity: 0.0; } }\n\
.rect { animation: fade 1s linear; }\n\
";

/// §5.4: "External engines may own: window; event loop; render graph; lifecycle;
/// frame callback." and "It returns render commands, pass descriptions or
/// textures through `RenderHost`/`OverlayHost`."
///
/// Case `5.4-a-render-without-kit-owned-window-or-loop`.
#[test]
fn a_frame_is_obtainable_without_a_kit_owned_window_or_event_loop() {
    let mut host = MockOverlayHost::new(800.0, 600.0);
    host.install_css(RECT_CSS);
    let tree = view(vec![el(1, "rect", vec![])]);

    // The host-driven frame entry, called exactly once. It returns; it does not
    // block in a loop.
    let result = host.drive_frame_at(0.0, &tree);

    assert!(
        result.composed(),
        "the host-driven frame entry returned a composed frame"
    );
    let commands = host.commands();
    assert!(
        !commands.is_empty(),
        "the call returns a non-empty batch of render commands"
    );

    let fills = host.fill_bounds();
    assert!(
        fills
            .iter()
            .any(|r| (r.size.width - 100.0).abs() < 0.01 && (r.size.height - 50.0).abs() < 0.01),
        "the batch contains a fill covering 100x50; got {fills:?}"
    );

    // The kit also hands back a texture through the host's own render target.
    let texture = host.render_to_target();
    assert!(
        !texture.is_empty(),
        "the kit returns a texture through the host-supplied render target"
    );

    assert_eq!(
        host.windows_created(),
        0,
        "zero windows exist in the process"
    );
    assert!(
        !host.event_loop_entered(),
        "the mock host's event loop was never entered"
    );
}

/// §5.4: "The kit receives: viewport; input; clock; render target; frame
/// scheduling hooks."
///
/// Case `5.4-b-host-clock-drives-time-not-wall-clock`.
#[test]
fn animation_time_follows_the_host_clock_not_the_wall_clock() {
    let mut host = MockOverlayHost::new(800.0, 600.0);
    host.install_css(FADE_CSS);
    let tree = view(vec![el(1, "rect", vec![])]);

    let wall_start = std::time::Instant::now();

    // Frame 1 at host clock 0 ms: the animation starts.
    let _ = host.drive_frame_at(0.0, &tree);

    // Frame 2 at host clock 500 ms, with no sleeping in between.
    let _ = host.drive_frame_at(500.0, &tree);

    let wall_elapsed = wall_start.elapsed();
    assert!(
        wall_elapsed < std::time::Duration::from_millis(5),
        "the two frames were driven with under 5 ms of wall time between them; \
         took {wall_elapsed:?}"
    );

    let half = host.layer_opacities();
    let sampled = *half.first().unwrap_or_else(|| {
        panic!("frame 2 paints the node through an opacity layer; got {half:?}")
    });
    approx(
        sampled,
        0.5,
        0.01,
        "frame 2 paints at 500/1000 of a linear 1.0 -> 0.0 ramp",
    );

    // A third frame at host clock 1000 ms lands at the ramp's end.
    let _ = host.drive_frame_at(1000.0, &tree);
    let end = host.layer_opacities();
    let sampled_end = *end
        .first()
        .unwrap_or_else(|| panic!("frame 3 paints the node through an opacity layer; got {end:?}"));
    approx(sampled_end, 0.0, 0.01, "the ramp ends at opacity 0.0");
}

/// §5.4: "The kit receives: … frame scheduling hooks." and "External engines may
/// own: … frame callback."
///
/// Case `5.4-c-frame-requested-through-host-scheduling-hook`.
#[test]
fn the_kit_asks_the_host_to_schedule_the_next_animation_frame() {
    let mut host = MockOverlayHost::new(800.0, 600.0);
    host.install_css(FADE_CSS);
    let tree = view(vec![el(1, "rect", vec![])]);

    // One frame at host clock 0 that starts a 1000 ms animation.
    let _ = host.drive_frame_at(0.0, &tree);

    assert!(
        host.frame_hook_calls() >= 1,
        "the kit invoked the host's frame-scheduling hook at least once during/after \
         the first frame, asking for another frame; got {} calls",
        host.frame_hook_calls()
    );

    let before = host.batch_fingerprint();

    // 200 ms of wall time during which the host drives nothing.
    std::thread::sleep(std::time::Duration::from_millis(200));

    let after = host.batch_fingerprint();
    assert_eq!(
        before, after,
        "no paint occurred and no render commands were produced while the host \
         drove no frame"
    );
}
