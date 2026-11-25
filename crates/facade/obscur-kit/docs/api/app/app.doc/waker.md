A waker that can turn the loop from another thread.

A panel spends nearly all its life asleep, and the news that should redraw it —
a clock, a battery reading, a subscription — arrives elsewhere. The worker
writes to shared state and rings this; the loop wakes and rebuilds the view from
whatever it finds.

Only meaningful alongside [`layer`](Self::layer): the windowed backend has its
own wake path.
