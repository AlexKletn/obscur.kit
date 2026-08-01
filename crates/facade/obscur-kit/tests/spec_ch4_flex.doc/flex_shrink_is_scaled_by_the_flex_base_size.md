css-flexbox-1 §9.7 Resolving Flexible Lengths:
"If using the flex shrink factor / For every unfrozen item on the line,
multiply its flex shrink factor by its inner flex base size, and note this as
its scaled flex shrink factor. Find the ratio of the item's scaled flex shrink
factor to the sum of the scaled flex shrink factors of all unfrozen items on
the line. Set the item's target main size to its flex base size minus a
fraction of the absolute value of the remaining free space proportional to the
ratio."
