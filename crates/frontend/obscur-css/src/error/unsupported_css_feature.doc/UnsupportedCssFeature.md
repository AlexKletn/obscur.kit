A CSS feature the kit does not support, reported as an explicit diagnostic
(spec §4.13).

Per §4.13, unsupported effects and fallbacks are made explicit rather than
silently dropped: when the parser meets a construct the kit cannot lower, it
emits one of these naming the offending token (for example `"@supports"`,
`"grid-template-areas"`, or `":has"`) and the [`SourceSpan`] it occupies. The
compiler can then continue, degrading gracefully instead of hard-failing.

It implements [`std::error::Error`] so it composes with the standard error
ecosystem while remaining I/O-free.
