The canvas painters the shipped widgets are drawn by.

A `<progress>` and a `<slider>` are not drawn by the toolkit's own paint
phase: each builds a `<canvas>` naming a painter, and the painter is an
ordinary registration on the runtime — the same door an application uses for
its own. Which means that until something registers them, the shipped widgets
draw *nothing at all*, silently, exactly like a canvas naming a painter
nobody supplied.

[`BuiltinPainters`] is the something. [`KitBootstrap::install_builtin_components`]
calls it on the same terms as the controls themselves, so every host that gets
the built-in widgets gets what draws them.

[`KitBootstrap::install_builtin_components`]: crate::bootstrap::KitBootstrap::install_builtin_components
