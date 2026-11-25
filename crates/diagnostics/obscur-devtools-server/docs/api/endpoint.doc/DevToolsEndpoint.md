A single authenticated DevTools session and the security policy around it.

The endpoint ties the [`Authenticator`], [`SubscriptionManager`], and
[`RuntimeBridge`] together and is the one place that enforces the §7.7
endpoint-security rules:

* **Disabled by default** — a fresh endpoint is not [`enable`](Self::enable)d
  and every handshake is rejected until the host turns it on.
* **Local-only, authenticated** — every session presents a token that the
  [`Authenticator`] must recognise; an unknown token is rejected.
* **Version-negotiated** — the server's supported [`VersionRange`] is
  negotiated against the client's; no overlap ⇒ rejection.
* **Scope-separated** — each command is authorised against the granted
  [`ScopeSet`] using its own required scope, so the four capability classes
  are independent.
* **Limit-enforcing** — subscriptions are bounded by the manager's limit.
* **Grant-honest** — a `Subscribe` is granted only when this session is
  actually being driven by a loop that pushes events (see
  [`services_event_push`](Self::services_event_push)); otherwise it is
  answered with a *denied* [`Subscription`] rather than a grant nothing
  will service. An `Inject` is the same rule in the other direction: it is
  [`ProtocolError::InjectionUnavailable`] unless the host installed a live
  [`InjectionSink`] to receive it (see
  [`with_injection_sink`](Self::with_injection_sink)).
* **Audited** — dangerous (`Control`/`Inject`) commands are appended to the
  audit log before they take effect — and before they are refused, so an
  attempt is recorded whether or not the host could service it.
* **No arbitrary execution** — control/injection resolve to typed
  acknowledgements only.

A production endpoint may be built [`with_control_excluded`](Self::with_control_excluded),
which turns every `Control` command into a [`ProtocolError::ControlDisabled`].
