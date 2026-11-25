Composes the function list into one affine when every distance is already a
number, and hands the list on unresolved when one is not.

The fold is kept here — rather than always deferring to computed value — so that
the overwhelmingly common absolute list costs exactly what it did before:
one [`PropertyValue::Transform`], no allocation, no computed-value work.
