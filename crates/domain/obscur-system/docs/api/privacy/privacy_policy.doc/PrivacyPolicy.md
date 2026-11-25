How privacy classes are enforced across diagnostics and capture (spec §7.12).

Safe by default: [`new`](Self::new) / [`Default`] redact sensitive data from
diagnostic snapshots and honour capture protection. Callers must explicitly
disable a guard to turn it off.
