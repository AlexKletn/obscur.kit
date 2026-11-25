The two backends put the ink of one opaque glyph in the same pixels.

The floor everything else in this file stands on: if the two disagreed about the
*shape* of an `H`, an alpha probe inside its stem would be meaningless. Compares
coverage past half rather than alphas, because the two rasterizers antialias
differently and are allowed to.

Needs an adapter, so it is `#[ignore]`d — counted as ignored, never as a pass —
and fails outright if it is asked for where no adapter exists.
