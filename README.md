# Obscur Kit

[Русская версия](README.ru.md) · [Contributing](CONTRIBUTING.md)

Obscur Kit is a Rust toolkit for building retained-mode user interfaces. Build a
view in Rust or author it in `.okui`, style it with CSS, and render it to a window
or off-screen. Applications use the [`obscur-kit` facade](crates/facade/obscur-kit/).

The runtime reconciles views into a retained tree, applies the cascade, computes
flow, flex and grid layout, and builds a scene for rendering. The toolkit also
provides controls, HarfRust text shaping, virtual lists, transitions, and
gestures with touchscreen and touchpad sources.

## Run an example

From the workspace root, run the windowed counter or render the capabilities
demo to PNG using the CPU backend.

```sh
cargo run -p obscur-kit --features full --example hello_app
cargo run -p obscur-kit --example capabilities_png
```

The windowed example needs a display and a GPU backend. Linux is the primary
desktop target. Wayland panels require a compositor that implements
`wlr-layer-shell`. Availability of native integrations depends on the enabled
Cargo features and the host platform.

## Application integration

- [Entry points](docs/api/01-entry-points.md) covers `App`, embedding the runtime,
  and headless use.
- [Input and gestures](docs/api/06-input.md) covers gesture sources and handling.
- [Hosting](docs/api/10-hosting.md) covers the host-facing APIs.
- The [facade manifest](crates/facade/obscur-kit/Cargo.toml) lists optional
  backends, including Vello, winit, layer shell, AccessKit, and process and WASM
  plugin execution. Applications define their own plugin integration points.

Build the Rust API reference with
`cargo doc --no-deps -p obscur-kit --features full`.

## Repository

- [`crates/`](crates/) contains the domain, runtime, frontend, components, adapters,
  plugin crates, and public facade.
- [`tools/`](tools/) contains the architecture checker, DevTools client, `.okui`
  language server, and plugin CLI.
- [`specs/`](specs/) contains the design and crate architecture. The dated
  [conformance audit](docs/conformance-audit.md) records a snapshot of coverage.

See the [contribution guide](CONTRIBUTING.md) for workspace checks and conventions.

## License

Project code is licensed under [MIT](LICENSE). The bundled Noto fonts use the
[SIL Open Font License 1.1](assets/fonts/LICENSE). Third-party material is
covered by its respective license notices.
