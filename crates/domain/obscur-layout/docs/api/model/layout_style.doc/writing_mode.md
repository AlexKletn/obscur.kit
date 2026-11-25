The block-flow / line orientation this box lays its in-flow content under
(`writing-mode`).

Like [`direction`](Self::direction) this is the box's own value as a
**containing block** for its in-flow children: css-writing-modes-4 §7.2
resolves flow-relative directions "with respect to the writing mode of the
containing block". [`FlowLayout`](crate::algorithm::FlowLayout) reads it to
decide which physical axis and origin it stacks children along:
[`WritingMode::HorizontalTb`] (the default) stacks top-to-bottom exactly as
before, [`WritingMode::VerticalRl`] stacks from the right edge leftward, and
[`WritingMode::VerticalLr`] stacks from the left edge rightward. Logical
box-model longhands (`margin-block-start` etc.) are already mapped to their
physical longhands upstream in the style crate under this same value.
