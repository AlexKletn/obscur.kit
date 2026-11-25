Whether the subscription was actually granted.

`false` means no event will ever arrive on it and the client should not
wait for one: the server denies rather than accepting a subscription it
cannot honour — today, when the session is being served request/reply and
nothing will push. A missing `Subscribe` scope and an exceeded
[`SUBSCRIPTION_LIMIT`] are refused earlier still, as a
[`ProtocolError`](crate::error::ProtocolError), so they never reach this
record.
