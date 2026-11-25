Returns a shared handle to the live winit window.

[`Self::winit`] borrows, which is enough to *drive* the window but not to
*present* to it: `wgpu::Instance::create_surface` needs an owned
`impl Into<SurfaceTarget<'static>>` to build a `Surface<'static>`, and
`Arc<Window>` is the handle that satisfies it. Handing out the [`Arc`]
rather than a borrow is what lets a host build its swapchain against a
window this backend created (spec §6.1).
