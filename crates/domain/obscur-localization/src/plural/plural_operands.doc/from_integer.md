The operands of an integer count — the overwhelmingly common case.

A negative count uses its absolute value: CLDR defines `n` as *the
absolute value of N*, so −1 and 1 are the same category. There are no
visible fraction digits, so `v = w = f = t = 0` — which is what makes
Russian's `v = 0` guard true and lets 1, 21, 101 reach `one` at all.
