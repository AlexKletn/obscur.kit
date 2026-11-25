Asks the host for a face installed under a family name nothing registered
carries, registering the answer for every later run.

This is the counterpart of [`FontRegistry::discover`]: that one asks *which
face covers this character*, this one asks *which face is called this*. Both
are needed, and only the second makes `font-family: "Figtree"` in an author
sheet mean anything on a machine where Figtree is installed — the application
never registered it, so the family loop above finds nothing and would
otherwise fall straight through to the next family.

Answers are keyed by the name the sheet asked for, and a name is put to the
host once: a face that came back is found in `discovered` from then on, and a
name that did not is remembered in `asked_families` so the miss costs one
lookup rather than one per run.
