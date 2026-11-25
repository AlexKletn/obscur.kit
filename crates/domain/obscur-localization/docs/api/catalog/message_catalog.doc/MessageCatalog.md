An in-memory table of patterns, keyed by locale and message key.

The reference [`MessageProvider`]: a host builds one at start-up from whatever
its translations live in, and hands it to a
[`Localizer`](crate::catalog::Localizer). It reports
[`SupportLevel::Native`] once it holds anything and
[`SupportLevel::Unavailable`] while empty — an empty catalogue and no
catalogue are the same capability, and saying otherwise would be the kind of
cheerful lie this workspace's ports are built to avoid.

Patterns are parsed on insert, not on lookup: [`insert`](Self::insert) returns
the [`MessagePatternError`] so a broken translation is caught when the
catalogue is loaded rather than on the frame that first needs it.
