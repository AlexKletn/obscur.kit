Creates a device-less Vello renderer with the given `id`.

The capability profile is the Vello GPU backend's: offscreen and MSAA
capable, custom shaders permitted (spec §5.3), HDR not offered. A host
must call [`Self::set_device`] before [`Renderer::render`] can submit GPU
work.
