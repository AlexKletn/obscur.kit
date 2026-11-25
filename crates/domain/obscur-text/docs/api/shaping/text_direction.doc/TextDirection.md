The base — or paragraph — writing direction of a text run (spec §5.10).

This is the *base direction* only: it seeds bidirectional layout but does not
itself encode per-character bidi levels. Full Unicode Bidirectional Algorithm
level resolution — splitting mixed left-to-right and right-to-left text into
runs and reordering them — is an adapter concern that happens during
itemization, before an individual run reaches the shaper; by the time a run is
shaped its direction is already uniform and known.
