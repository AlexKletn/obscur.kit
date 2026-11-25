How the user may leave a kiosk or presentation session (spec §6.5).

`SafeEscapePolicy` is a tightly-coupled part of [`PresentationMode`](super::PresentationMode): it only
has meaning as the escape rule of a presentation policy, so it is colocated
here rather than given its own module. The default is
[`SafeEscapePolicy::Allow`], the safe choice that never traps the user.
