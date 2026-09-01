Registers a face under `family`, the name an author sheet selects it by.

```no_run
# use obscur_kit::app::App;
const FIGTREE: &[u8] = &[];
App::new("Keys")
    .font("Figtree", FIGTREE)
    .stylesheet("body { font-family: Figtree, sans-serif; }");
```

Without this an application had no way to name a font at all: the only faces
a runner installed were the host's three generics, so every
`font-family: "X"` in a sheet resolved to nothing and *installing* X changed
nothing either. A face given here is registered before the host's, so it wins
a tie against a same-named family the machine happens to have.

Bytes are a whole `.ttf`/`.otf` file — vendored with `include_bytes!`,
downloaded, or read from the application's own data directory. For a face
that is a particular cut (a real bold, a real italic), use
[`font_face`](Self::font_face); this one registers the regular upright cut,
and asking for a weight it does not carry gets a synthesized one.
