Returns the next frame id.

The counter advances by one, wrapping on overflow so the operation is
total and can never panic. In practice a `u64` frame clock never reaches
the wrap point during any real runtime session.
