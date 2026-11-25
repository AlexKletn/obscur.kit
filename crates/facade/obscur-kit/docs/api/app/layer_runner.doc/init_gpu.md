A Vello renderer and a configured surface for this layer window.

Asks for a premultiplied alpha mode where the compositor offers one, so a
translucent panel reads through to whatever is behind it — which for a bar over
a blurred desktop is the whole visual point.

The instance, surface and adapter come from `gpu_instance::open`, which asks for
the first-tier backends before the whole set. That is not a preference, it is a
measurement: bringing up the GLES backend alongside Vulkan cost this process
58 MB of resident memory and four `/dev/dri` handles, for a backend that on this
Wayland layer surface answers `gl not compatible with provided surface`.
