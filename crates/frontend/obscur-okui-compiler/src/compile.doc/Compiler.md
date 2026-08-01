Runs the full `.okui` compilation pipeline over one parsed document,
producing UI IR and a unified diagnostic stream (spec §2.9–§2.15).

The compiler is configured fluently with the component contracts
([`with_registry`](Self::with_registry)), the granted capabilities
([`granting`](Self::granting)), the commands a handler may name
([`claiming_command`](Self::claiming_command)) and the props/state symbols in
scope ([`defining`](Self::defining)) before [`compile`](Self::compile) is
called.
