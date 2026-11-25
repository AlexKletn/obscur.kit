Returns whether `name` is claimed by a declared command.

Always `true` on an [`Unchecked`](Self::Unchecked) surface: with nothing
declared, no name can be shown to be unclaimed, and refusing one would be
the false error this type exists to avoid.
