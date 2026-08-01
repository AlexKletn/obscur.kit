How [`PrivacyClass`] tags are enforced across capture and diagnostics
(spec §6.17, §7.12).

[`PrivacyPolicy`] is the safe-by-default configuration the runtime consults:
sensitive data is redacted from diagnostic snapshots and capture-protected
data is excluded from screen capture unless a caller explicitly opts out
(spec §7.12). It answers two questions — should this class be redacted, and
should it be excluded from capture — given a [`PrivacyClass`].
