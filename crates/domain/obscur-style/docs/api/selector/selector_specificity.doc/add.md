Combines two specificities component-wise, saturating on overflow.

Used to accumulate the specificity of a compound or complex selector
from the specificities of its parts. Saturation means a pathologically
long selector clamps at [`u16::MAX`] per component rather than wrapping,
preserving the "more is never less" ordering guarantee.
