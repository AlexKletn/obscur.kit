Points the runtime's layout viewport at the swapchain's physical extent.
The scene is rasterized 1:1 into the surface, so laying out under a
different viewport would leave the content in a corner. Called after the
surface is created and on every resize. Unlike the old host, the runtime
carries retained state, so this just updates the viewport in place (no
rebuild); the caller requests a frame to actually re-lay-out.
