The Levenshtein distance between `left` and `right`, counted over `char`s.

The textbook two-row dynamic program: `O(len × len)` time, `O(len)` space, over
characters rather than bytes so a non-ASCII handler name is not measured in
UTF-8 code units.
