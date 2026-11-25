One `DrawText` command carrying the same half-alpha `H` twice at the same place.

The live divergence's input: the CPU backend fills each glyph separately (¾),
Vello unions them inside one run (½).
