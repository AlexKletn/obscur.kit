Gives this endpoint somewhere to deliver an authorized `Inject`: the push
end of an [`InjectionQueue`](crate::transport::InjectionQueue) the host
holds and drains (spec §7.6).

Without one, `Inject` is refused with
[`ProtocolError::InjectionUnavailable`] — see [`dispatch`](Self::dispatch).

# This grants nothing

A sink is a *destination*, not a permission. The `Inject` scope check runs
first and is untouched by this: a session without the scope is refused on
the scope whether or not a sink exists, and installing a sink can only ever
move an authorized `Inject` from "refused" to "delivered" — never the
reverse, and never past the [`Authenticator`], which in a
[`production`](Authenticator::production) build has already stripped
`Inject` from every grant (§7.7).
