Compiles the UA default stylesheet into a runtime-matchable [`Stylesheet`] at
the [`CascadeOrigin::SystemDefault`] origin — the lowest in the cascade, so any
author/theme declaration overrides it (§4.14).

Hand the result to
[`UiRuntime::set_default_stylesheet`](crate::runtime::UiRuntime::set_default_stylesheet)
(or let
[`KitBootstrap::install_default_stylesheet`](crate::bootstrap::KitBootstrap::install_default_stylesheet)
do it) to give an app the toolkit's overridable base component look.
