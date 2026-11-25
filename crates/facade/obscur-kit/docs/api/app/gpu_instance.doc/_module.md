Opening the one GPU instance the process runs on, without opening the ones it
does not use.

`wgpu::Instance` is not one connection. `wgpu_core::Instance::new` calls
`try_add_hal` once per backend compiled in, and `InstanceDescriptor::new_without_display_handle()`
asks for `Backends::all()` — so on this workspace's Linux build (Vulkan and
GLES; dx12 and metal compile but their cfgs are Windows- and Apple-only, noop
is off) a default instance brings up **both**, and `create_surface` then makes a
surface on each one that came up.

Measured on this desktop, the shipped keyboard, three runs of each, six seconds
apiece:

```text
                              /dev/dri handles   VmRSS (3 runs)         frames/presented
Backends::all() (before)      7  (6x renderD128  112.3 / 112.8 / 112.9  2/2 2/2 2/2
                                  + 1x card1)
WGPU_BACKEND=vulkan           3  (2x renderD128   53.9 /  53.9 /  54.1  2/2 2/2 2/2
                                  + 1x card1)
PRIMARY first (this file)     3  (2x renderD128   54.1 /  53.9 /  53.9  2/2 2/2 2/2
                                  + 1x card1)
```

Same frames, same presents, no panics, `libgallium`/`libEGL` gone from the
process map entirely.

Roughly **58 MB of resident memory and four DRM handles** for a backend that
never draws anything: with GLES present the process also maps `libEGL`,
`libEGL_mesa` and the whole of `libgallium-26.2`, and none of it is touched
because the adapter chosen is the Vulkan one either way.

The two middle rows were taken while this module still read `WGPU_BACKEND`; it
no longer does (`attempts.md` says why, and it is a fault this module
introduced), so the second of them can no longer be reproduced from a build of
this file. The first and last rows can: they are the difference between the
`Instance::default()` the runners used to say and what they say now.

And on the surface this program actually puts on screen, GLES could not have
drawn anything even if it had been asked to. Asked for on its own, on this
desktop, wgpu answered:

```text
No suitable graphics adapter found; ... gl not compatible with provided surface ...
```

which is the descriptor's `display: None` meeting a Wayland surface — wgpu's own
`InstanceDescriptor::display` documentation says the GLES backend needs the
display handle "especially for Wayland". So the 58 MB bought a backend that
cannot present here, not a fallback.

It is still kept as the second attempt, because "cannot present *here*" is a
statement about this desktop and this surface, and the toolkit is not only this
program. So: ask for the first-tier backends first, and fall back to the whole
set only if that finds no adapter. A machine with no usable Vulkan reaches GLES
exactly as before, one failed `request_adapter` later — verified by blinding the
Vulkan ICD (`VK_DRIVER_FILES=/nonexistent`), where the second attempt does run
and the error that comes back is the same all-backends message the
single-instance code produced. That variable still works, and with
`WGPU_BACKEND` gone it is now the only way to exercise the fallback from
outside. A machine with Vulkan stops paying for a driver stack it never uses.
