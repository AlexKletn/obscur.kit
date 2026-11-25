Applies the §1.7 error-boundary decision for `failure` against
`boundary`, returning the [`RecoveryResult`].

The sequence is:
1. Record the failure in the boundary. It is *always* recorded — even a
   disabled boundary stores it — so it can be reported to diagnostics.
2. If the boundary did not contain it (disabled), the failure escapes:
   [`RecoveryResult::Propagated`].
3. Otherwise, if the boundary still has retry budget, retry:
   [`RecoveryResult::Retried`].
4. Otherwise the fallback UI is kept in place of the failed subtree:
   [`RecoveryResult::FellBack`].
