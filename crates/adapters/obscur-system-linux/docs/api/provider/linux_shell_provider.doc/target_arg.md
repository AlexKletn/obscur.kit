Extracts the string handed to the opener for `target` (spec §6.11).

A [`ShellTarget::Uri`] contributes the URI, a [`ShellTarget::Document`] its
opaque token, and a [`ShellTarget::File`] its advisory path hint when present,
falling back to the handle token otherwise.
