Extracts a verified native package and negotiates its version-1 lifecycle table.
Module constructors can run as soon as the loader opens the library. The
library remains mapped for the process lifetime, including on negotiation error.

# Safety

The host must trust the exact binary and all of its native dependencies to
execute in-process, and must guarantee that the exported version-1 symbol and
callbacks implement the documented ABI without unwinding across C. The host
must protect the extraction parent and its ancestors from untrusted changes
and allocate a handle appropriate for this instance.
