The blit's load op is pinned by a test, so moving it is a deliberate act with
a measurement attached rather than an edit that nobody notices.

This is a tripwire, not a proof: it cannot tell whether `Load` is right, only
that whoever changed it meant to. The proof — such as it is, on the one
machine that can be asked — is `the_two_load_ops_draw_the_same_pixels_and_here_is_what_each_costs`
and the numbers written into `LOAD_OP.md`.
