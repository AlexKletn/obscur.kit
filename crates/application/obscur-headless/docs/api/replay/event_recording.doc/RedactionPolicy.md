Which classes of sensitive input are redacted when recorded (spec §7.13).

Redaction is **default-on**: [`RedactionPolicy::default`] redacts both
secrets and protected clipboard data. Redaction is also **irreversible** —
when a class is redacted the original payload is dropped, never stored, so a
recording can never leak a redacted secret no matter how it is later
inspected or replayed.
