Returns whether `current` reports a different configuration than `previous`
for the same display (spec §6.6).

The comparison is exactly what [`DisplayEvent::ConfigurationChanged`]
documents — mode, scale, or bounds — plus the work area, which is the
reserved-region half of bounds. Primacy is excluded because it has its own
[`DisplayEvent::PrimaryChanged`], and reporting a primary flip twice would
make a consumer that re-lays-out on a configuration change do so for a change
that moved no geometry. The name is excluded because it is the identity this
display was matched by in the first place.
