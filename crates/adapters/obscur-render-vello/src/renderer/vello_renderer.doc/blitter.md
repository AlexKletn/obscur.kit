Cached surface blitter for the on-screen present path, keyed by the
surface format it was built for. Rebuilding a `TextureBlitter` recompiles
its render pipeline, so we keep one alive across frames and only rebuild
when absent or when the surface format changes.
