The wgpu device, cached so resizes can reconfigure the surface without
reaching into the renderer (a `wgpu::Device` is a cheap Arc handle).
