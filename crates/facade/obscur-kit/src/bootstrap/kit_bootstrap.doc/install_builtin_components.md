Registers the shipped controls (spec §2.2) on `runtime`, unless this
bootstrap was built [`without_builtin_components`].

**This is on by default, and it changes what existing markup renders.** An
authored `<checkbox label="Agree" checked/>` stops being an empty element
with the right id and becomes the control: the `.checkbox-box` indicator
the UA sheet fills, plus the label. `App` and the layer-shell runner call
it for you, on the same terms as the UA stylesheet — a standalone window is
the "batteries included" entry point, and a control tag that renders nothing
is the defect this closes. A host that embeds a `UiRuntime` itself calls it
(or does not) explicitly.

It is idempotent in the only sense that matters: calling it twice returns
[`RegistryError::DuplicateComponent`] rather than replacing anything.

[`without_builtin_components`]: KitBootstrap::without_builtin_components
[`RegistryError::DuplicateComponent`]: crate::view::component::RegistryError::DuplicateComponent

# Errors

[`RegistryError::DuplicateComponent`] when a built-in's id is already
claimed on `runtime`. Opting out never fails: it registers nothing.
