Returns whether this range shares at least one version with `other`.

Two inclusive ranges overlap when each starts no later than the other
ends: `self.min <= other.max && other.min <= self.max`.
