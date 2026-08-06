Turns the text of a `style` attribute into declarations, and into whatever it
had to report to recover them.

Installed by whoever owns the CSS front end — the application tier cannot depend
on a frontend crate (§1.11), so the grammar arrives as a function rather than as
a type. An unparseable declaration is dropped and the rest of the block kept;
the [`InlineStyleOutcome`](super::InlineStyleOutcome) carries both halves, so
the drop is reported rather than silent.
