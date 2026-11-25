Everything both sides of the handover need to agree about, behind one lock.

It was split before: the image lived in a mutex the acquire thread could reach
and the "is one on the way" flag lived as a plain `bool` on the loop's side of
the struct. Two halves of one fact, and only one of them was shared, so the loop
could — and did — take back a claim about an acquire that had not stopped
running. Both halves are here now, under the same lock, which is the only place
a claim about the acquire thread can be true.
