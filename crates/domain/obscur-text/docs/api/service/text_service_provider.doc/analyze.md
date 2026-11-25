Analyzes `request` and returns its diagnostics (spec §5.13).

The returned [`TextDiagnostic`]s are typed, ranged (in **document** byte
offsets), and stamped with [`request.version()`](LanguageServiceRequest::version).
An implementation MUST:

- only run a service the request asks for
  ([`request.wants`](LanguageServiceRequest::wants)); and
- skip any issue whose position falls in an ignored / sensitive range
  ([`request.is_ignored`](LanguageServiceRequest::is_ignored)).

Asynchronous and incremental delivery is an adapter concern; here the
contract is the pure mapping request → diagnostics.
