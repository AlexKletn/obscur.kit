A 200×120 `overflow: auto` scroller holding six 200×40 rows, every row bound to
one click command.

240 px of content in a 120 px scrollport is 120 px of scroll, and every scroll
assertion in the suite is written against those numbers: a reveal of the last row
scrolls to exactly 120, a coast that would travel 325 px arrives at 120 and
stops. One command across all six rows is enough because the question these tests
ask is never *which* row — it is whether a gesture that scrolled also activated
something, which it must not.

The associated functions serve tests that render their own list:
[`view`](Self::view) for unbound rows, [`with_rows`](Self::with_rows) to decorate
each row by index, [`sheet`](Self::sheet), and [`nodes`](Self::nodes).
