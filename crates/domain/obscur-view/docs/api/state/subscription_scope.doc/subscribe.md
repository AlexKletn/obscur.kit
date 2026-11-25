Records a dependency on `source`, which was at generation `observed` when
the dependency was taken, returning the subscription's freshly minted id.

The scope keeps ownership of the token; the returned id lets the caller
refer to it later (for example to [`unsubscribe`](Self::unsubscribe)).
