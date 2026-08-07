The used font size, as raw bits so the key compares by exact bit pattern
rather than by `f32` equality (which has no `Eq`, and would make `NaN`
mismatch itself).
