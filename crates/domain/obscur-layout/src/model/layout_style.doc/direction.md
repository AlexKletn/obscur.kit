The inline base direction this box lays its in-flow content along
(`direction`).

This is the box's own value when it acts as a **containing block** for its
in-flow children: css-writing-modes-4 §7.2 resolves flow-relative
directions "with respect to the writing mode of the containing block", and
names the over-constrained-margin case as its worked example — "the margin
that is dropped when a box's inline dimension is over-constrained is the end
margin as determined by the writing mode of the containing block"
(css-writing-modes-4.txt:1988-1990). [`FlowLayout`](crate::algorithm::FlowLayout)
reads this to decide which physical edge an over-constrained in-flow child
settles against: [`Direction::Ltr`] (the default) keeps the child's
line-left (`margin-left`) edge, [`Direction::Rtl`] keeps its line-right
(`margin-right`) edge so the child lands against the content box's right
side. Logical box-model properties (`margin-inline-start` etc.) are already
mapped to their physical longhands upstream in the style crate; this field
carries only the geometric half the style crate cannot express.
