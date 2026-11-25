//! The present blit, with the multiply a premultiplied surface expects.

#[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/LOAD_OP.md")]
const LOAD_OP: wgpu::LoadOp<wgpu::Color> = wgpu::LoadOp::Load;

#[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/PremultiplyBlitter.md")]
pub(super) struct PremultiplyBlitter {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    #[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/bound.md")]
    bound: Option<(wgpu::TextureView, wgpu::BindGroup)>,
}

impl PremultiplyBlitter {
    #[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/new.md")]
    #[cfg(test)]
    pub(super) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self::with_alpha_conversion(device, format, true)
    }

    pub(super) fn with_alpha_conversion(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        premultiply: bool,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("obscur-premultiply-blit"),
            source: wgpu::ShaderSource::Wgsl(include_str!("premultiply_blit.wgsl").into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("obscur-premultiply-blit"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("obscur-premultiply-blit"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("obscur-premultiply-blit"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(if premultiply {
                    "fragment"
                } else {
                    "fragment_copy"
                }),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                // No blending: this pass REPLACES the surface, it does not
                // composite onto it. The compositor is what blends, and the
                // whole point of the shader is to hand it the form it blends.
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("obscur-premultiply-blit"),
            // The source and the target are the same extent on every present, so
            // the filter never actually interpolates; `Nearest` says so rather
            // than leaving a resample nobody asked for available.
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..wgpu::SamplerDescriptor::default()
        });
        Self {
            pipeline,
            layout,
            sampler,
            bound: None,
        }
    }

    #[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/bind_source.md")]
    fn bind_source(&mut self, device: &wgpu::Device, source: &wgpu::TextureView) {
        // The key is the source view itself, kept beside the group it was built
        // from. `wgpu::TextureView`'s `Eq` proxies to the handle underneath, so
        // this compares OBJECT IDENTITY — not a descriptor, not an extent. Two
        // views of the same texture with the same descriptor are still two
        // views, and still a miss, which is the safe direction to err in.
        //
        // Keying it rather than invalidating it is the point: there is no site
        // that has to remember to call a `clear()`, so no site can forget. The
        // full list of things that can make this group wrong, and where each is
        // answered:
        //
        // 1. The source view is replaced — the surface resized, so
        //    `VelloRenderer::cached_offscreen` built a new `VelloFrame`. Answered
        //    here, by the comparison below: a different view is a miss.
        // 2. The surface format changed — `VelloRenderer` drops the whole
        //    blitter (`vello_renderer.rs`, the `self.blitter = None` above
        //    `get_or_insert_with`), and the cache is a field of it, so it dies
        //    with the layout and sampler it was built against.
        // 3. The layout, the sampler or the pipeline changed — they are built
        //    once in `new` and never mutated; changing one means a new blitter,
        //    which is case 2.
        // 4. The device was lost — today `VelloDevice` is built once per process
        //    and never rebuilt (see `vello_renderer.doc/frame_cache.md`, which
        //    records that invariant and what would have to be reset with it).
        // 5. The TARGET changed — every present, because every acquire is a new
        //    swapchain texture. It is not in this group at all: the target is
        //    the render pass's attachment, not a binding. This is the one that
        //    changes most often and the one that cannot invalidate the cache.
        // 6. The source's PIXELS changed — every frame, by construction. A bind
        //    group binds a view, not its contents; this is not an invalidation.
        //
        // The one cost of keeping it: the entry holds a clone of the source
        // view, which holds its texture. After a resize the previous offscreen
        // therefore outlives its replacement until the next `copy` swaps the
        // entry — one frame, one texture, and never more than one, because the
        // miss below replaces rather than appends.
        if self.bound.as_ref().is_none_or(|(bound, _)| bound != source) {
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("obscur-premultiply-blit"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            self.bound = Some((source.clone(), group));
        }
    }

    #[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/copy.md")]
    pub(super) fn copy(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
    ) {
        self.bind_source(device, source);
        let (_, bind_group) = self
            .bound
            .as_ref()
            .expect("bind_source leaves a group bound");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("obscur-premultiply-blit"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: LOAD_OP,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::PremultiplyBlitter;

    /// The one format this crate's blit is ever built for on a wlr layer
    /// surface, and the one this test reads back through.
    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
    const EXTENT: u32 = 4;

    #[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/NO_ADAPTER.md")]
    const NO_ADAPTER: &str = "no GPU adapter on this machine, and these tests were \
        asked for by name. They check a device resource, so there is nothing they \
        could have verified without one and passing here would be a lie. Either run \
        them where an adapter exists, or leave them ignored — the default — where \
        they are counted as ignored and not as passed.";

    #[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/gpu.md")]
    fn gpu() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::default();
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect(NO_ADAPTER);
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("the adapter this machine reported refused to open a device")
    }

    /// A single-colour source texture, straight alpha, and a view onto it.
    fn source(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba: [u8; 4],
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("blit-test-source"),
            size: wgpu::Extent3d {
                width: EXTENT,
                height: EXTENT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let pixels: Vec<u8> = rgba.iter().copied().cycle().take(4 * 4 * 16).collect();
        queue.write_texture(
            texture.as_image_copy(),
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(EXTENT * 4),
                rows_per_image: Some(EXTENT),
            },
            wgpu::Extent3d {
                width: EXTENT,
                height: EXTENT,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// Blit `view` onto a fresh target and read the top-left texel back.
    fn blit_and_read(
        blitter: &mut PremultiplyBlitter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
    ) -> [u8; 4] {
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("blit-test-target"),
            size: wgpu::Extent3d {
                width: EXTENT,
                height: EXTENT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        // 4 px × 4 B = 16 B a row, and wgpu wants rows padded to 256.
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("blit-test-readback"),
            size: 256 * u64::from(EXTENT),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        blitter.copy(device, &mut encoder, view, &target_view);
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256),
                    rows_per_image: Some(EXTENT),
                },
            },
            wgpu::Extent3d {
                width: EXTENT,
                height: EXTENT,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(std::iter::once(encoder.finish()));
        readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let data = readback.slice(..).get_mapped_range();
        let first = [data[0], data[1], data[2], data[3]];
        drop(data);
        readback.unmap();
        first
    }

    #[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/a_second_source_is_what_the_second_blit_draws.md")]
    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test -p obscur-render-vello --lib -- --ignored`"]
    fn a_second_source_is_what_the_second_blit_draws() {
        let (device, queue) = gpu();
        let mut blitter = PremultiplyBlitter::new(&device, FORMAT);

        // Opaque red, then opaque blue, through two DIFFERENT views. A cache
        // that never invalidates draws red twice and fails on the second read;
        // one keyed by anything the two share — the extent, the format, the
        // descriptor — does the same.
        let (_red, red_view) = source(&device, &queue, [255, 0, 0, 255]);
        let first = blit_and_read(&mut blitter, &device, &queue, &red_view);
        assert_eq!(
            first,
            [255, 0, 0, 255],
            "the first blit draws the red source"
        );

        let (_blue, blue_view) = source(&device, &queue, [0, 0, 255, 255]);
        let second = blit_and_read(&mut blitter, &device, &queue, &blue_view);
        assert_eq!(
            second,
            [0, 0, 255, 255],
            "the second blit draws the SECOND source; a bind group cached past \
             the view it was built from would still be drawing the first"
        );
    }

    #[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/the_same_source_is_bound_once.md")]
    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test -p obscur-render-vello --lib -- --ignored`"]
    fn the_same_source_is_bound_once() {
        let (device, queue) = gpu();
        let mut blitter = PremultiplyBlitter::new(&device, FORMAT);
        let (_green, view) = source(&device, &queue, [0, 255, 0, 255]);

        blitter.bind_source(&device, &view);
        let first = blitter
            .bound
            .as_ref()
            .expect("the first bind leaves a group")
            .1
            .clone();
        blitter.bind_source(&device, &view);
        let second = &blitter.bound.as_ref().expect("the group survives").1;

        // Identity, not equality: this is the whole claim of the patch. Were the
        // group rebuilt per present the cache would be an allocation with extra
        // steps, and this assertion is what says it is not.
        assert_eq!(&first, second, "the same source view keeps the same group");
    }

    #[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/the_alpha_is_multiplied_in.md")]
    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test -p obscur-render-vello --lib -- --ignored`"]
    fn the_alpha_is_multiplied_in() {
        let (device, queue) = gpu();
        let mut blitter = PremultiplyBlitter::new(&device, FORMAT);
        // Straight alpha in: full red at half alpha. Premultiplied out: half red.
        let (_half, view) = source(&device, &queue, [255, 0, 0, 128]);
        let read = blit_and_read(&mut blitter, &device, &queue, &view);
        assert_eq!(read[3], 128, "alpha passes through untouched");
        assert!(
            (127..=129).contains(&read[0]),
            "colour is scaled by alpha, got {read:?}"
        );
        // And the reason `LoadOp::Clear` is safe: the target is written whole,
        // so nothing of what was in it survives to be blended with. A texel the
        // triangle missed would read as the clear colour, not as this.
        assert_eq!(read[1], 0, "nothing but the source reaches the target");
    }

    #[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/the_load_op_is_the_one_that_was_measured.md")]
    #[test]
    fn the_load_op_is_the_one_that_was_measured() {
        assert_eq!(
            super::LOAD_OP,
            wgpu::LoadOp::Load,
            "the blit's load op is a MEASUREMENT, not a preference. `Load` is \
             what this desktop measured faster; `Clear` is what a tiled GPU is \
             expected to prefer and what nothing here can weigh. Before moving \
             this line, run `cargo test -p obscur-render-vello --lib \
             the_two_load_ops -- --ignored --nocapture` on the machine the \
             change is for, and put its numbers in LOAD_OP.md."
        );
    }

    /// A `width` x `height` attachment plus a source of the same size, and the
    /// per-pass wall cost of blitting one onto the other under a given load op.
    ///
    /// Passes are batched `PER_SUBMIT` to a submit and the queue is drained
    /// once per batch, so what is divided out is GPU work and not submit
    /// latency. Blocks alternate so a machine drifting under load drifts
    /// through both arms equally.
    #[cfg(test)]
    fn time_load_op(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        blitter: &mut PremultiplyBlitter,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
        batches: u32,
    ) -> f64 {
        const PER_SUBMIT: u32 = 50;
        blitter.bind_source(device, source);
        let (_, bind_group) = blitter.bound.as_ref().expect("bind_source binds one");
        let mut best = f64::MAX;
        for _ in 0..batches {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            for _ in 0..PER_SUBMIT {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("load-op-bench"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_pipeline(&blitter.pipeline);
                pass.set_bind_group(0, bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
            let started = std::time::Instant::now();
            queue.submit(std::iter::once(encoder.finish()));
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            let per_pass = started.elapsed().as_secs_f64() / f64::from(PER_SUBMIT);
            best = best.min(per_pass);
        }
        best * 1e6
    }

    #[doc = include_str!("../../docs/api/renderer/premultiply_blit.doc/the_two_load_ops_draw_the_same_pixels_and_here_is_what_each_costs.md")]
    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test -p obscur-render-vello --lib the_two_load_ops -- --ignored --nocapture`"]
    fn the_two_load_ops_draw_the_same_pixels_and_here_is_what_each_costs() {
        // The panel this program exists to draw on, in physical pixels: the
        // tablet's 2944x1004. The load op's cost is per attachment texel, so
        // measuring it at 4x4 would measure nothing.
        const WIDTH: u32 = 2944;
        const HEIGHT: u32 = 1004;

        let (device, queue) = gpu();
        let mut blitter = PremultiplyBlitter::new(&device, FORMAT);

        let src = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("load-op-source"),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let row: Vec<u8> = [255u8, 0, 0, 128]
            .iter()
            .copied()
            .cycle()
            .take(WIDTH as usize * 4)
            .collect();
        let pixels: Vec<u8> = row
            .iter()
            .copied()
            .cycle()
            .take(WIDTH as usize * HEIGHT as usize * 4)
            .collect();
        queue.write_texture(
            src.as_image_copy(),
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(WIDTH * 4),
                rows_per_image: Some(HEIGHT),
            },
            wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
        );
        let source_view = src.create_view(&wgpu::TextureViewDescriptor::default());

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("load-op-target"),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

        // The assertion first, because it is the claim the choice rests on: the
        // triangle covers the whole attachment and the pipeline blends nothing,
        // so what was in the target before the pass cannot reach the result.
        // Dirty the target, blit under each op, and compare the full readback.
        let junk = read_after(
            &device,
            &queue,
            &mut blitter,
            &source_view,
            &target,
            &target_view,
            wgpu::LoadOp::Clear(wgpu::Color::GREEN),
            true,
        );
        let loaded = read_after(
            &device,
            &queue,
            &mut blitter,
            &source_view,
            &target,
            &target_view,
            wgpu::LoadOp::Load,
            false,
        );
        assert_eq!(
            junk, loaded,
            "the two load ops must produce identical pixels — the pass writes \
             every texel, so the loaded content is never read"
        );

        // Warm up: the first submits pay for pipeline compilation and residency.
        let _ = time_load_op(
            &device,
            &queue,
            &mut blitter,
            &source_view,
            &target_view,
            wgpu::LoadOp::Load,
            2,
        );

        let mut load = Vec::new();
        let mut clear = Vec::new();
        for _ in 0..3 {
            load.push(time_load_op(
                &device,
                &queue,
                &mut blitter,
                &source_view,
                &target_view,
                wgpu::LoadOp::Load,
                4,
            ));
            clear.push(time_load_op(
                &device,
                &queue,
                &mut blitter,
                &source_view,
                &target_view,
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                4,
            ));
        }
        let fmt = |v: &[f64]| {
            v.iter()
                .map(|x| format!("{x:.1}"))
                .collect::<Vec<_>>()
                .join(" / ")
        };
        let median = |v: &mut Vec<f64>| {
            v.sort_by(|a, b| a.partial_cmp(b).expect("no NaN from a clock"));
            v[v.len() / 2]
        };
        let (lo, cl) = (fmt(&load), fmt(&clear));
        let (lm, cm) = (median(&mut load), median(&mut clear));
        println!(
            "load op, {WIDTH}x{HEIGHT} attachment, us per pass (best batch of 4, 50 passes a submit):\n  \
             Load  {lo}  median {lm:.1}\n  \
             Clear {cl}  median {cm:.1}\n  \
             Clear - Load = {:+.1} us a present on this adapter",
            cm - lm
        );
    }

    /// Fill the target with junk if asked, blit under `load`, read it all back.
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    fn read_after(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        blitter: &mut PremultiplyBlitter,
        source: &wgpu::TextureView,
        target: &wgpu::Texture,
        target_view: &wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
        dirty_first: bool,
    ) -> Vec<u8> {
        let width = target.width();
        let height = target.height();
        let padded = (width * 4).div_ceil(256) * 256;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("load-op-readback"),
            size: u64::from(padded) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        blitter.bind_source(device, source);
        let (_, bind_group) = blitter.bound.as_ref().expect("bind_source binds one");
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        if dirty_first {
            // A pass that only clears, so the NEXT pass has something loud to load.
            drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("load-op-dirty"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 1.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            }));
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("load-op-blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&blitter.pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(std::iter::once(encoder.finish()));
        readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let data = readback.slice(..).get_mapped_range();
        let out = data.to_vec();
        drop(data);
        readback.unmap();
        out
    }
}
