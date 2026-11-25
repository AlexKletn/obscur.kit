Builds the default raster-image decoder (the `image`-crate adapter that
turns encoded PNG/JPEG bytes into RGBA8 pixels + intrinsic size, spec
§5.16).

Pair it with [`Self::default_resource_loader`] and inject both into a
[`UiRuntime`](crate::runtime::UiRuntime) via
[`set_resource_providers`](crate::runtime::UiRuntime::set_resource_providers)
(or use [`Self::install_image_resources`]) to make `background-image:
url(...)` resolve against the filesystem.
