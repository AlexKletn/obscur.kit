The broad category of a prop's declared Rust type, kept on the tooling
surface so a string-keyed checker can reject a literal of the wrong kind at
compile time (spec §2.8, "typed props … compile-time validation").

`#[derive(Contract)]` classifies each prop field's type into one of these
from its normalized type string; the `.okui` compiler then compares it
against the literal an attribute was given (`label={5}` → an integer for a
`String` prop). [`Other`](PropTypeSurface::Other) is the escape hatch for a
type the classifier does not recognise (an enum, a struct, a newtype), and a
prop of that kind is never type-checked — the surface cannot know what
literal, if any, would be valid.
