Narrows a 0-based `usize` index into the 1-based `u32` line number `index + 1`,
saturating so a pathologically large grid cannot overflow.
