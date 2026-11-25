The `style` attribute: a declaration block written on the element, which the
cascade admits at origin `Inline` (§4.14), above every stylesheet and below a
runtime override.

Reserved rather than hashed because `StyleAttributes::extract` reads this exact
symbol. Under a hashed one the text would be carried through reconciliation and
then never compiled — the same failure `placeholder` had.
