A single line-break opportunity: a byte offset in the source text where a
line may (soft) or must (hard) break (spec §5.10).

The [`offset`](Self::offset) is a **byte** index into the UTF-8 source and
marks the position at which the following text would start a new line. Breaks
order by `offset` first, then by [`kind`](Self::kind) (soft before hard),
which the derived [`Ord`] provides.
