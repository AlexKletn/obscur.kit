Registers a new subscription to `topic`, returning its fresh id.

# Errors

Returns [`ProtocolError::LimitExceeded`] when the endpoint already holds
[`SUBSCRIPTION_LIMIT`] subscriptions (spec §7.7).
