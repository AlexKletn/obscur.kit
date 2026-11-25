Dispatches an authenticated command, enforcing the §7.7 security core.

The steps, in order: authorise the command's
[`required_scope`](CommandMessage::required_scope) against the granted
[`ScopeSet`] (a session with no scopes — including one that never
handshook — fails here); audit the command if it is dangerous; refuse
`Control` when control is excluded; then route to the bridge, the
subscription manager, or a typed acknowledgement.

# Errors

Returns [`ProtocolError::Unauthorized`] when the granted scopes lack the
required scope, [`ProtocolError::ControlDisabled`] when a control command
is issued to a control-excluded endpoint,
[`ProtocolError::InjectionUnavailable`] when an authorized `Inject` has no
live [`InjectionSink`] to be delivered to, and
[`ProtocolError::LimitExceeded`] when a subscription would exceed the
limit.
