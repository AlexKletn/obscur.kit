The cases both backends must answer identically, each with a probe point and
the answer the domain requires.

`Clip::Mask` is the regression this test was written for. The domain settles
which answer is correct: a `Mask` "does not carry the source itself"
(`mask.rs`) and `Clip::bounds()` is `None` for a mask because "a mask's extent
depends on its source content, which is not known to the clip itself"
(`clip.rs`). A mask clip therefore states no extent, and a backend may not
invent one — least of all the empty extent, which throws the content away.
