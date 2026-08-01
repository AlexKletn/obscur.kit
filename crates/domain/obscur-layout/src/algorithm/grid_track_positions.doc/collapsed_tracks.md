Marks which tracks of an `auto-fit` repeat region collapse: a track inside the
`[start, end)` generated range that no placed item covers. Returns an
all-`false` vector (nothing collapses) when there is no `auto-fit` region — so
an `auto-fill` or plain template is untouched.
