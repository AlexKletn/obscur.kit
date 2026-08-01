A 400-wide row holding one `width: 50%` item — so the item is 200 — which is
itself a wrapping row of two 60px boxes. Two 60s fit on one line at 200 and
wrap at 100, so the item's height says which number it believed its
containing block was. `align-items: flex-start` is what routes the item down
the *measured* cross-size path rather than the stretch.
