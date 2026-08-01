Alignment distributes the **content** box's slack, not the border box's: a
padded container aligns its text to the inside of the padding.

This is the case that says which width is actually being used. `width` is the
*content* width here (content-box sizing), so a 300px box with 20px of padding
either side has its content spanning 20..320 and a right-aligned line ends at
**320**. Were the padding not subtracted from the border box the alignment
width would be 340 and the line would end at 360, well outside the box.
