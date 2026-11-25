Drains every pending request most-urgent-first, emptying the queue.

The returned order is deterministic: it matches successive [`pop`]s.

[`pop`]: SchedulerQueue::pop
