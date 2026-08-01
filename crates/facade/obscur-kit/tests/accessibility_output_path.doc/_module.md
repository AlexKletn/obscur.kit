Spec §7.1: the semantic tree's output path — an ordinary frame reaches an
AccessKit client.

The audit's §7-A closed the *builder* (a disabled control stopped being
announced as enabled) and left the finding that outlived the fix: a correct
semantic tree was computed and published to nobody, because `AccessKitBridge`
was never constructed outside its own crate and `push_accessibility`'s only
caller was a test. These tests drive the real frame path with a real AccessKit
bridge installed and assert what the bridge was actually handed — a tree that
merely got *built* would satisfy nothing here.

The bridge is a workspace-unconditional dev-dependency rather than the facade's
`accesskit` feature, so this runs under a plain `cargo test --workspace`.
