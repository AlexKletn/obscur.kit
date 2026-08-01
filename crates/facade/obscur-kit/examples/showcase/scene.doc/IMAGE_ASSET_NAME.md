The file name the image cards reference. Both `background-image: url(...)` and
the `<img>` [`Image`] primitive resolve this against the resource loader's
sandbox root, so an example writes [`image_asset_bytes`] to
`<sandbox>/gallery.png` before rendering.
