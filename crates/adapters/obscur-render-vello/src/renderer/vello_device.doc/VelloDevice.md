A wgpu device/queue plus the [`vello::Renderer`] built on top of them
(spec §2.7).

# Ownership of the GPU context

wgpu adapter/device creation is asynchronous and requires a real GPU and an
event loop, which the adapter deliberately does **not** own. The *host*
creates the [`wgpu::Device`] and [`wgpu::Queue`] (it holds the async/windowing
context) and hands them to [`VelloDevice::new`], which then builds the Vello
renderer synchronously. This keeps the adapter free of any
`Instance::request_adapter` call, so the crate compiles and unit-tests run
without a GPU (spec §5.2 determinism boundary — only rasterization is
device-bound).

The renderer is initialised with the full anti-aliasing support set
([`AaSupport::all`]) so any [`vello::AaConfig`] chosen at render time is
available, `use_cpu` disabled (GPU rasterization), and no pipeline cache.
