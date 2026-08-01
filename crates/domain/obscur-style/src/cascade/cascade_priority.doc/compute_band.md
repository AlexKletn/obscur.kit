Computes the precedence band from `(origin, important)` (§4.14).

* The runtime-override layer is the top band regardless of importance;
  within it, `!important` ranks above normal.
* Otherwise an authored `!important` declaration lands in a band above
  all authored normal declarations.
* Otherwise an authored normal declaration's band is its origin rank.
