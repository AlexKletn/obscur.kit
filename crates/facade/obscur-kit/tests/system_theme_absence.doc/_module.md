What happens on a machine with no desktop appearance to read.

The answer is: **nothing bad**. No `Result`, no panic, no error — an absent
desktop is a legitimate machine, and an application there is unstyled by the
system rather than broken by it. What it does get is one warning, because
silence would make "my colours did not follow the desktop" look like a defect in
the application.

These tests pin both halves: that it degrades, and that it says so at no more
than warning level.
