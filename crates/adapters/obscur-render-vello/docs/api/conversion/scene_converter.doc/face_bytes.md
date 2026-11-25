The bytes registered under a run font handle, for tests that need to prove
something about the allocation rather than about the drawing.

Test-only and `pub(crate)`: `VelloRenderer::register_font` is where the sharing
decision is made and where the test that pins it lives, and that is a different
module from this one.
