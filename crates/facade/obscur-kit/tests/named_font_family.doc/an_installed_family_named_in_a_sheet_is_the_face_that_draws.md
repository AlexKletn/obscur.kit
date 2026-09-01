A family the host installs, asked for by name, is looked up and drawn in —
and the face pulled in is the file listed under that family.

The subject is the machine's own serif, so the test asserts nothing about
which fonts a box has: whatever `fc-match serif` names is installed by
construction, covers Latin, and — the point — was registered under the
*generic* `serif` and never under its own name.

The bytes assertion is the one that matters. `fc-match` answers every query,
including for families that do not exist, so a lookup built on it would return
the system default under the author's name and this test would pass while the
interface drew in the wrong font.
