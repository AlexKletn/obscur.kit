How many lines of its own `impl` make a candidate a *main* entity.

Measured, not chosen: across the workspace this threshold separates 4 files
from 190 whose second `pub` type is vocabulary rather than an entity (see the
module docs). Sixty lines is roughly "more than a constructor, an accessor
and a `Default`" — below it a type is described by the entity that owns it,
above it the type describes itself.
