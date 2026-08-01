What the per-frame lowering costs, and the claim it backs: keeping the **IR** and
lowering it again is cheap, where re-parsing the source per frame — the obvious
wrong answer — is not.

The numbers are printed (`cargo test -- --nocapture`) because a wall-clock
threshold is not a property worth asserting on someone else's machine. What *is*
asserted is the ratio the design rests on: a lowering costs a fraction of a
parse+compile, so a screen can be rebuilt every frame and a source cannot.
