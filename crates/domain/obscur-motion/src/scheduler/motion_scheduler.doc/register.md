Registers `timeline` into the active set, keyed by its
[`id`](Timeline::id).

The timeline is inserted **as-is**: its own lifecycle state governs
advancement, so the caller is responsible for having
[`start`](Timeline::start)ed it. Registering a timeline whose id already
exists replaces the previous one.
