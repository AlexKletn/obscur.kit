A handle that makes a sleeping UI loop turn once.

A panel spends nearly all its time asleep — that is the point of it — and the
news that should redraw it usually arrives on another thread: a clock ticking,
a battery reading, a subscription firing. Without this the loop would have to
poll, which is the difference between a panel that costs nothing and one that
does not.

`Send + Sync`, because the thread that has the news is by definition not the
one that owns the loop. Each platform's loop has its own way of being woken;
this is the one word an application needs to know.
