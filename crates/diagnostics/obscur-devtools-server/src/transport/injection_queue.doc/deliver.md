Delivers `event` to the host's queue.

Returns `false` when there is nothing to deliver to — the host's
[`InjectionQueue`] has been dropped — and the event is discarded. The
endpoint turns that `false` into a refusal rather than an acknowledgement.

A `true` return with a full queue still means delivered: the oldest event
is evicted and counted in [`dropped`](InjectionQueue::dropped). Refusing
the command would be wrong there — the channel is working, the host is
behind — and the count is where that is reported.
