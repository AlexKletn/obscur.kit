Runs a scheduler-gated reactive frame, reads the committed scene, and
presents it on-screen via the Vello offscreen-render + blit path. A lost
swapchain is reconfigured and the frame skipped (the next redraw recovers).
