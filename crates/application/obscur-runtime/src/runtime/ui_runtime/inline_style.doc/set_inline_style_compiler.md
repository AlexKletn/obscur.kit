Installs the thing that turns a `style` attribute's text into declarations.

This crate does not parse CSS and does not depend on the front end that does,
so the compiler arrives from whoever wired the CSS pipeline —
`KitBootstrap` does it for you. Until one is installed a `style` attribute is
INERT rather than half-understood.
