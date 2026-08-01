Creates a loader confined to `root`: every `File` path resolves relative
to `root` and is rejected if it escapes the canonicalized root subtree
(spec §5.15 file, §2.7).
