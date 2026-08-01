The out-of-workspace consumer check: does the toolkit work from the far side
of the facade?

`checks/facade-consumer` is a crate whose only dependency is `obscur-kit`, so
its extern prelude holds `obscur_kit` and nothing else. That is the one place
a defect in what the *authoring surfaces emit* can be seen: an `okui!`
expansion naming `::obscur_view` compiles in every crate of this workspace and
in no application anywhere (audit finding 1-A).

It cannot be a workspace member — the facade-dependency rule forbids that, and
membership would put the internal crates back in the graph — so `cargo test
--workspace` does not reach it and this command exists to.
