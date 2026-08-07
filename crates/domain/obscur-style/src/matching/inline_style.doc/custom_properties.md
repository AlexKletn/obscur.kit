The custom properties (`--name: value`) the block declared, in the order
they were written. Held apart from the declarations for the same reason a
[`StyleRule`](crate::matching::StyleRule) holds them apart: they are keyed
by name, not by [`PropertyId`](crate::property::PropertyId), and they
cascade a step before every longhand does.
