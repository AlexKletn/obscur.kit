Re-syncs the runtime's viewport + device scale to the current swapchain
extent and display density (called after surface creation and on resize).

The viewport is the swapchain's **physical** extent divided by the device
scale, so layout runs in **logical** CSS pixels that follow the real window
size and reflow on resize.
