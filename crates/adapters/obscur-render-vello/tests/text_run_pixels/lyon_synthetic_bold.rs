#![doc = include_str!("../text_run_pixels.doc/lyon_draws_synthetic_bold_without_fallback.md")]

use super::{FACE, NO_ADAPTER, NoFallback, PaintCommand, VelloDevice, VelloRenderer, raster, text};

#[doc = include_str!("../text_run_pixels.doc/lyon_draws_synthetic_bold_without_fallback.md")]
#[test]
#[ignore = "needs a GPU adapter"]
fn lyon_draws_synthetic_bold_without_fallback() {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect(NO_ADAPTER);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("open GPU device");
    let mut classic = VelloRenderer::default()
        .with_device(VelloDevice::new(device.clone(), queue.clone()).expect("classic device"));
    let mut lyon = VelloRenderer::default().with_lyon_device(device, queue);
    lyon.set_diagnostic_sink(std::sync::Arc::new(NoFallback));
    for renderer in [&mut classic, &mut lyon] {
        renderer.register_font(0, FACE.to_vec());
    }

    let commands = |oblique, bold| {
        let PaintCommand::DrawText { origin, run, brush } =
            text(&[(36, 0.0), (3, 28.0), (50, 36.0)], 36.0)
        else {
            unreachable!("text helper returns DrawText");
        };
        vec![PaintCommand::DrawText {
            origin,
            run: run.with_synthesis(oblique, bold),
            brush,
        }]
    };
    let coverage =
        |pixels: &[u8]| -> u32 { pixels.chunks_exact(4).map(|rgba| u32::from(rgba[3])).sum() };
    for oblique in [false, true] {
        let regular = raster(&mut lyon, commands(oblique, false));
        let bold = raster(&mut lyon, commands(oblique, true));
        let reference = raster(&mut classic, commands(oblique, true));
        assert!(coverage(&bold) > coverage(&regular));
        assert!(
            coverage(&bold).abs_diff(coverage(&reference)) < coverage(&reference) / 10,
            "bold coverage differs from Vello: Lyon {}, Vello {}",
            coverage(&bold),
            coverage(&reference),
        );
        assert_eq!(raster(&mut lyon, commands(oblique, false)), regular);
    }
}
