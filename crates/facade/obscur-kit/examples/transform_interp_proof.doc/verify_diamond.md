Asserts the painted region is the rotate(45°) **diamond** at full scale:
* the centre is inked,
* the topmost inked pixel in the centre column sits ≈ `50·√2 ≈ 70.7px` above
  the centre (the diamond's top vertex — proving rotation AND scale
  preservation; a 50px axis-aligned box or a shrunk lerp could not reach it),
* the un-rotated corner `C + (50, 50)` is OUTSIDE the diamond, hence clear.
