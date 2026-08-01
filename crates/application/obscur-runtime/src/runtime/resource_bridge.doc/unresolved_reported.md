Image sources this runtime has already reported as unresolvable because no
loader/decoder is installed (spec §5.15 "The kit does not perform arbitrary
network access by itself."). Deduplicates the diagnostic so a referenced
URL is named once, not once per frame.
