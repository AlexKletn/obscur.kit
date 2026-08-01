The total-order sort key used to pick the winning declaration for a property
(§4.14).

The greater priority wins. Priority is derived from a declaration's
`(origin, important, specificity, source_order)` and imposes the §4.14
ordering as three bands, from lowest to highest precedence:

1. **authored normal** declarations, ordered among themselves by origin
   (`SystemDefault < Theme < GlobalApp < ComponentModule < Inline`);
2. **authored `!important`** declarations, in the same origin order — this
   whole band sits above *every* authored normal declaration, so an
   important Theme declaration beats a normal Inline one;
3. **runtime-override** declarations, which sit above all authored CSS,
   normal or important; within this top band an `!important` runtime
   override outranks a normal one.

These three regions are captured by a single [`band`](Self::band) integer.
The complete ordering key is the tuple `(band, specificity, source_order)`:
within one band (which pins down a single `(origin, important)` pairing) a
higher [`SelectorSpecificity`] wins, and on equal specificity a later
`source_order` wins.

Because [`band`](Self::band) is a bijection over the valid
`(origin, important)` inputs, ordering by that tuple is consistent with
equality: two priorities compare equal exactly when their band, specificity
and source order all match.
