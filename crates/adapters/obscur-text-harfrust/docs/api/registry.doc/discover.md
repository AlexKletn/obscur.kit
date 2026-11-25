Resolves `text` against the host lookup: finds the first character no
registered face covers, asks for a face that does, and keeps it.

Returns the newly discovered font's index in `discovered`, or [`None`]
when there is no lookup, every character is already covered by something,
or the host has nothing to offer.
