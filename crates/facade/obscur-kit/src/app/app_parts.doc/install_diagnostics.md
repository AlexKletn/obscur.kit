Install the chosen sink, or a default that is not silence.

The application's choice wins. With no choice, a **debug** build gets
[`StderrDiagnostics`](crate::bootstrap::StderrDiagnostics) and a release build
keeps the runtime's silent default.

Runs first in the boot, before the stylesheets are compiled: that compile is the
loudest producer here, and its complaints — a dropped declaration, an
unsupported function — are the ones an application most needs to hear.
