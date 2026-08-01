Returns `true` if `self` is strictly newer than `other`.

Consumers use this against their last-observed generation to detect a
pending change without diffing values.
