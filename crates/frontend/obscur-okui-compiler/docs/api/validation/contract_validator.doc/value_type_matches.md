Whether a value of type `found` satisfies a prop declared `expected`.

An [`Unknown`](AttributeValueType::Unknown) value and an
[`Other`](PropTypeSurface::Other) prop are always accepted — neither carries
enough information to prove a mismatch. An integer literal is accepted for a
float prop (`opacity={1}`), since it is a valid float; every other pairing
must match exactly.
