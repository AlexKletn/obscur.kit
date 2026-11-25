The winit [`ApplicationHandler`] behind [`App::run`](super::App::run): it owns
the window, the GPU-backed Vello renderer, the swapchain surface, and the
embeddable [`UiRuntime`], and composes the kit's own facilities
([`KitBootstrap`], [`CssPipeline`], the winit converters) rather than
hand-wiring them again. Lifted from the `uikit_window` / `windowed_counter`
examples so an application does not spell out ~700 lines to open one window.
