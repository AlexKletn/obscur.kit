As reportable as the *least* reportable source being read.

`Native` only when every source this provider would consult can also report a
change. `Limited` when some can and some cannot — the ordinary KDE-with-a-portal
case, where the scheme is reported and the palette is not. `Unavailable` when
none can.

Deliberately the minimum rather than the maximum, which is the opposite of
[`support`](Self::support) and for the opposite reason: there, more sources can
only add to what is known; here, one silent source is enough to make the whole
answer late.
