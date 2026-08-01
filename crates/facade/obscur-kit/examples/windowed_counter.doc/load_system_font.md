Loads a system font's raw bytes for the renderer's text path.

The toolkit never bundles a font (fonts are a host resource, like the GPU
device), so the example reads one from a small list of well-known locations
across Linux/macOS/Windows and returns the bytes of the first that exists.
Returns `None` when none are present, in which case text simply does not
render.
