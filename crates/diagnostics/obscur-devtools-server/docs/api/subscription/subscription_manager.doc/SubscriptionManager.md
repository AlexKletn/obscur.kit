Tracks active subscriptions per [`Topic`] and fans pushed events out to
their subscribers.

The manager mints monotonically increasing [`SubscriptionId`]s and enforces
the [`SUBSCRIPTION_LIMIT`]: an endpoint may hold at most that many concurrent
subscriptions before [`subscribe`](Self::subscribe) fails with
[`ProtocolError::LimitExceeded`] (spec §7.7). Records are kept in insertion
order so fan-out is deterministic.
