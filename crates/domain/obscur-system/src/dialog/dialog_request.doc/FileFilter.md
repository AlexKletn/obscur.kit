A named set of file extensions constraining a file dialog (spec §6.9).

A filter pairs a human-readable `name` (for example `"Images"`) with the
extensions it matches (for example `["png", "jpg"]`). Extensions are stored
verbatim without a leading dot; interpretation is the provider's concern.
