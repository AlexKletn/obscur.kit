Installs `sources` through the real CSS pipeline, renders one frame of
`root`, and returns the runtime beside its inspection snapshot.

One pipeline for all the sources, exactly as an application would install
them: the sheets have to agree about what `--brand` means without ever
having met.
