Byte offsets of *collapsible* trailing-hangable whitespace (CSS Text 3
§3/§7.1): a run of these at a line's end is removed from the line's used
width (it hangs). Only the collapsing modes have collapsible spaces
(normal / nowrap / pre-line); pre / pre-wrap preserve them, so nothing
hangs there and their measurement is untouched.
