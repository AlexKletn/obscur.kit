Indexes `sheets` by the decoration each rule could produce.

A rule is kept for a target exactly when a selector of that rule could match
it — the same subject-compound test the matcher itself applies — so a rule this
misses is a rule that could never have contributed a declaration.
