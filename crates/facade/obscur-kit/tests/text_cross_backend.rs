#![doc = include_str!("text_cross_backend.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_geometry::{Point, Size};
use obscur_render::frame::{FrameSubmission, RenderFrame};
use obscur_render::target::{BitmapTarget, RenderTarget};
use obscur_render::{Renderer, RendererId};
use obscur_render_cpu::renderer::CpuRenderer;
use obscur_render_vello::conversion::SceneConverter;
use obscur_render_vello::renderer::{VelloDevice, VelloRenderer};
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::paint::Brush;
use obscur_scene::scene::{SceneGeneration, SceneSnapshot};
use obscur_scene::text::{GlyphRun, PositionedGlyph};
use vello::peniko::{Blob, FontData};

#[doc = include_str!("text_cross_backend.doc/FACE.md")]
const FACE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../assets/fonts/NotoSans-Regular.ttf"
));

#[doc = include_str!("text_cross_backend.doc/WIDTH.md")]
const WIDTH: u32 = 128;
#[doc = include_str!("text_cross_backend.doc/HEIGHT.md")]
const HEIGHT: u32 = 64;

#[doc = include_str!("text_cross_backend.doc/ORIGIN.md")]
const ORIGIN: Point = Point::new(8.0, 50.0);

#[doc = include_str!("text_cross_backend.doc/SIZE.md")]
const SIZE: f32 = 40.0;

#[doc = include_str!("text_cross_backend.doc/CAPITAL_H.md")]
const CAPITAL_H: u32 = 43;

#[doc = include_str!("text_cross_backend.doc/PROBE.md")]
const PROBE: (u32, u32) = (12, 22);

#[doc = include_str!("text_cross_backend.doc/HALF.md")]
const HALF: f32 = 0.5;

#[doc = include_str!("text_cross_backend.doc/ONCE.md")]
const ONCE: u8 = 128;
#[doc = include_str!("text_cross_backend.doc/TWICE.md")]
const TWICE: u8 = 192;
#[doc = include_str!("text_cross_backend.doc/SLACK.md")]
const SLACK: u8 = 2;

#[doc = include_str!("text_cross_backend.doc/NO_ADAPTER.md")]
const NO_ADAPTER: &str = "no GPU adapter on this machine, and this test was asked \
    for by name. It compares pixels a GPU produced against pixels the CPU backend \
    produced, so there is nothing it could have verified without one and passing \
    here would be a lie. Either run it where an adapter exists, or leave it \
    ignored — the default — where it is counted as ignored and not as passed.";

#[doc = include_str!("text_cross_backend.doc/text.md")]
fn text(glyphs: &[(u32, f32)], alpha: f32) -> PaintCommand {
    PaintCommand::DrawText {
        origin: ORIGIN,
        run: GlyphRun::new(
            glyphs
                .iter()
                .map(|(id, x)| PositionedGlyph::new(*id, 0, *x, 0.0, SIZE * 0.6))
                .collect(),
            SIZE,
            "H",
        ),
        brush: Brush::solid(Color::rgba(1.0, 1.0, 1.0, alpha)),
    }
}

#[doc = include_str!("text_cross_backend.doc/submission.md")]
fn submission(commands: &[PaintCommand]) -> FrameSubmission {
    let snapshot = SceneSnapshot::new(
        DisplayList::from_commands(commands.to_vec()),
        SceneGeneration::from_raw(1),
    );
    #[allow(clippy::cast_precision_loss)]
    FrameSubmission::new(RenderFrame::new(
        snapshot,
        Size::new(WIDTH as f32, HEIGHT as f32),
    ))
}

#[doc = include_str!("text_cross_backend.doc/cpu_alpha.md")]
fn cpu_alpha(commands: &[PaintCommand]) -> Vec<u8> {
    let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
    renderer.register_font(0, FACE.to_vec());
    #[allow(clippy::cast_precision_loss)]
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(WIDTH as f32, HEIGHT as f32)));
    renderer
        .render(&submission(commands), &target)
        .expect("the CPU backend renders the text target");
    let pixmap = renderer.pixmap().expect("a frame was rendered");
    pixmap.pixels().iter().map(|p| p.alpha()).collect()
}

#[doc = include_str!("text_cross_backend.doc/vello_alpha.md")]
fn vello_alpha(renderer: &mut VelloRenderer, commands: &[PaintCommand]) -> Vec<u8> {
    let (_, _, bytes) = renderer
        .render_to_image(&submission(commands), WIDTH, HEIGHT)
        .expect("the Vello backend rasterizes the text target");
    bytes.chunks_exact(4).map(|p| p[3]).collect()
}

#[doc = include_str!("text_cross_backend.doc/at.md")]
fn at(alpha: &[u8], point: (u32, u32)) -> u8 {
    alpha[(point.1 * WIDTH + point.0) as usize]
}

#[doc = include_str!("text_cross_backend.doc/glyph_runs.md")]
fn glyph_runs(commands: &[PaintCommand]) -> usize {
    let mut converter = SceneConverter::new();
    converter.register_font(0, FontData::new(Blob::from(FACE.to_vec()), 0));
    converter
        .convert(&DisplayList::from_commands(commands.to_vec()))
        .encoding()
        .resources
        .glyph_runs
        .len()
}

#[doc = include_str!("text_cross_backend.doc/vello.md")]
fn vello() -> VelloRenderer {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("the adapter this machine reported refused to open a device");
    let device = VelloDevice::new(device, queue).expect("a Vello device on that adapter");
    let mut renderer = VelloRenderer::new(RendererId::from_raw(2)).with_device(device);
    renderer.register_font(0, FACE.to_vec());
    renderer
}

#[doc = include_str!("text_cross_backend.doc/inked.md")]
fn inked(alpha: &[u8]) -> usize {
    alpha.iter().filter(|a| **a > ONCE).count()
}

#[doc = include_str!("text_cross_backend.doc/disagreements.md")]
fn disagreements(left: &[u8], right: &[u8]) -> usize {
    assert_eq!(left.len(), right.len(), "two rasters of the same target");
    left.iter()
        .zip(right)
        .filter(|(l, r)| (**l > ONCE) != (**r > ONCE))
        .count()
}

#[doc = include_str!("text_cross_backend.doc/two_commands.md")]
fn two_commands() -> Vec<PaintCommand> {
    vec![
        text(&[(CAPITAL_H, 0.0)], HALF),
        text(&[(CAPITAL_H, 0.0)], HALF),
    ]
}

#[doc = include_str!("text_cross_backend.doc/one_command_two_glyphs.md")]
fn one_command_two_glyphs() -> Vec<PaintCommand> {
    vec![text(&[(CAPITAL_H, 0.0), (CAPITAL_H, 0.0)], HALF)]
}

#[doc = include_str!(
    "text_cross_backend.doc/each_text_command_composites_on_its_own_on_both_backends.md"
)]
#[test]
fn each_text_command_composites_on_its_own_on_both_backends() {
    // The CPU half, in pixels and without a device: two half-alpha commands over
    // the same glyph must reach 1-(1-½)² = ¾, not ½. That is the CPU backend
    // saying, measurably, that a `DrawText` is its own composite.
    let cpu = at(&cpu_alpha(&two_commands()), PROBE);
    assert!(
        cpu.abs_diff(TWICE) <= SLACK,
        "the CPU backend composited two half-alpha text commands to {cpu}/255, not \
         {TWICE}/255. Either it stopped compositing per command, or the probe \
         {PROBE:?} is no longer inside the glyph."
    );

    // The Vello half, structurally: the encoding must hold one glyph run per
    // command, because a run is one path and one draw object. Two runs is two
    // source-over composites and matches the CPU number above; one run would be
    // a single non-zero fill of the union, which is ½, and the two backends
    // would be drawing different pictures from the same list.
    let runs = glyph_runs(&two_commands());
    assert_eq!(
        runs, 2,
        "the Vello backend encoded {runs} glyph run(s) for 2 text commands. A run \
         is one path and one draw object, so folding commands into one replaces N \
         source-over composites with one non-zero fill of their union — while the \
         CPU backend, measured just above, still composites once per command. \
         See tests/text_cross_backend.doc/_module.md."
    );
}

#[doc = include_str!("text_cross_backend.doc/overlapping_glyphs_composite_once.md")]
#[test]
fn overlapping_glyphs_composite_once() {
    let commands = one_command_two_glyphs();
    let single = [text(&[(CAPITAL_H, 0.0)], HALF)];
    let cpu = cpu_alpha(&commands);
    assert_eq!(cpu, cpu_alpha(&single));
    assert_eq!(at(&cpu, PROBE), ONCE);
    assert_eq!(glyph_runs(&commands), 1);
}

#[doc = include_str!("text_cross_backend.doc/overlapping_bold_glyphs_composite_once.md")]
#[test]
#[ignore = "needs a GPU adapter; run with `cargo test -p obscur-kit --test text_cross_backend -- --ignored`"]
fn overlapping_bold_glyphs_composite_once() {
    let mut renderer = vello();
    let commands = one_command_two_glyphs()
        .into_iter()
        .map(synthetic_bold)
        .collect::<Vec<_>>();
    let single = [synthetic_bold(text(&[(CAPITAL_H, 0.0)], HALF))];

    let cpu = cpu_alpha(&commands);
    assert_eq!(cpu, cpu_alpha(&single));
    assert_eq!(at(&cpu, PROBE), ONCE);

    let gpu = vello_alpha(&mut renderer, &commands);
    assert_eq!(at(&gpu, PROBE), ONCE);
    // Vello's antialiasing can change edge coverage with winding magnitude.
    assert!(gpu.iter().all(|&alpha| alpha <= ONCE));
}

#[doc = include_str!("text_cross_backend.doc/the_backends_paint_the_same_text.md")]
#[test]
#[ignore = "needs a GPU adapter; run with `cargo test -p obscur-kit --test text_cross_backend -- --ignored`"]
fn the_backends_paint_the_same_text() {
    let mut renderer = vello();
    let opaque = vec![text(&[(CAPITAL_H, 0.0)], 1.0)];
    let cpu = cpu_alpha(&opaque);
    let gpu = vello_alpha(&mut renderer, &opaque);

    // Coverage, not colour: the two rasterizers antialias differently and are
    // allowed to (the same allowance `clip_cross_backend` makes). What must
    // agree is which pixels the glyph covers, and the outline of a 40 px 'H' is
    // ~300 pixels of which only the edge is in doubt.
    let (a, b) = (inked(&cpu), inked(&gpu));
    let off = disagreements(&cpu, &gpu);
    eprintln!("opaque 'H': cpu ink {a}, vello ink {b}, {off} pixels on opposite sides of half");
    assert!(
        off * 20 < a.min(b),
        "the two backends disagree about {off} of ~{} covered pixels — far more \
         than antialiasing on the outline of one glyph can account for",
        a.min(b)
    );
}

#[doc = include_str!("text_cross_backend.doc/the_backends_composite_two_commands_the_same.md")]
#[test]
#[ignore = "needs a GPU adapter; run with `cargo test -p obscur-kit --test text_cross_backend -- --ignored`"]
fn the_backends_composite_two_commands_the_same() {
    let mut renderer = vello();
    let commands = two_commands();
    let cpu = cpu_alpha(&commands);
    let gpu = vello_alpha(&mut renderer, &commands);

    let (c, g) = (at(&cpu, PROBE), at(&gpu, PROBE));
    eprintln!("two half-alpha commands at {PROBE:?}: cpu {c}/255, vello {g}/255");
    assert!(
        c.abs_diff(TWICE) <= SLACK && g.abs_diff(TWICE) <= SLACK,
        "two half-alpha text commands over the same glyph must reach {TWICE}/255 on \
         both backends; got cpu {c}, vello {g}. A backend reading ~{ONCE}/255 has \
         folded the two commands into one fill of their union."
    );

    // And the same claim across the whole raster, not just one pixel: nowhere may
    // one backend be past half coverage while the other is not.
    let off = disagreements(&cpu, &gpu);
    let floor = inked(&cpu).min(inked(&gpu));
    eprintln!("two half-alpha commands: {off} pixels on opposite sides of half, ink floor {floor}");
    assert!(
        off * 20 < floor,
        "the two backends disagree about {off} of ~{floor} covered pixels when the \
         same two text commands are composited"
    );
}

#[doc = include_str!("text_cross_backend.doc/synthetic_bold_adds_the_same_weight.md")]
#[test]
#[ignore = "needs a GPU adapter; run with `cargo test -p obscur-kit --test text_cross_backend -- --ignored`"]
fn synthetic_bold_adds_the_same_weight() {
    let mut renderer = vello();
    let regular = [text(&[(CAPITAL_H, 0.0)], 1.0)];
    let bold = [synthetic_bold(regular[0].clone())];
    let coverage = |alpha: Vec<u8>| alpha.into_iter().map(i64::from).sum::<i64>();
    let cpu_gain = coverage(cpu_alpha(&bold)) - coverage(cpu_alpha(&regular));
    let gpu_gain = coverage(vello_alpha(&mut renderer, &bold))
        - coverage(vello_alpha(&mut renderer, &regular));
    assert!(
        cpu_gain > 0 && gpu_gain > 0,
        "both glyph outlines must thicken"
    );
    assert!(
        cpu_gain.abs_diff(gpu_gain) * 8 <= cpu_gain.min(gpu_gain) as u64,
        "synthetic bold coverage gain differs between CPU ({cpu_gain}) and Vello ({gpu_gain})"
    );
}

#[doc = include_str!("text_cross_backend.doc/synthetic_bold.md")]
fn synthetic_bold(mut command: PaintCommand) -> PaintCommand {
    let PaintCommand::DrawText { run, .. } = &mut command else {
        unreachable!("fixture requires a DrawText command");
    };
    *run = run.clone().with_synthesis(false, true);
    command
}
