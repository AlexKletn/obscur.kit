Offers `diagnostic` to this subscription, returning whether it was
accepted into the buffer.

A diagnostic is rejected when the subscription is disabled or the
diagnostic does not pass the filter. Otherwise it is buffered under the
policy: at capacity, the [`OverflowStrategy`] decides whether the oldest
entry is evicted, the newcomer is dropped, or the diagnostic is
coalesced with a matching buffered entry.
