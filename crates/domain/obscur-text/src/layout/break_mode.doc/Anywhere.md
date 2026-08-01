`anywhere`: like [`BreakWord`](Self::BreakWord) for the greedy line
breaker — a last-resort grapheme-cluster break to avoid overflow. (It also
differs from `break-word` in how it affects min-content intrinsic sizing,
which the greedy wrapper does not distinguish.)
