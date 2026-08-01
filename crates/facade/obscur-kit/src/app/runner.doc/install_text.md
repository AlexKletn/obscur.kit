Installs the real text stack over the host's system fonts (bundled last-resort
fallback appended by [`install_text_stack`](KitBootstrap::install_text_stack)),
and hands the renderer the *same* id→bytes assignment so scene font handles
line up with what it rasterizes — the codified fix for the id-matching problem.
