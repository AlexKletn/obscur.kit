A [`FontResolver`] that answers "the first font in the chain that covers the
*whole* run", which is what css-fonts-4 §5.3 asks for: "If all characters in
the sequence b + c1 + c2 … are completely supported by the font, select this
font for the sequence."

It additionally implements the canonical-equivalence rule: "If a sequence of
multiple codepoints is canonically equivalent to a single character and the
font supports that character, select this font for the sequence."
