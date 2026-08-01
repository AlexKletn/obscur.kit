The Rust condition expression for one branch.

A missing `cond` on an `<If>` / `<ElseIf>` emits `false`: a condition that was
never stated is not true, so the branch does not render. The alternative —
rendering a branch whose condition is absent — is the silent-wrong-output
behaviour this whole path exists to remove.
