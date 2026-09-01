Ask to be told when the appearance changes.

`None` where this source cannot report changes — check
[`watch_support`](Self::watch_support) first if the answer matters, since it is
the difference between "poll instead" and "something went wrong".

The watch lasts as long as the returned
[`AppearanceWatch`](super::AppearanceWatch); dropping it stops.
