Declares `bytes` as a face of `family` with `attributes`.

The bytes are not parsed here — [`install_text_stack`](super::KitBootstrap::install_text_stack)
parses them once, and bytes that are not a loadable font simply never cover
a run (they are not an error, and never a panic).
