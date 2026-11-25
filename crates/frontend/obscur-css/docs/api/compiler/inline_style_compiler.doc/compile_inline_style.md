Compiles the text of one `style` attribute.

The synthetic selector's specificity is discarded: what comes back is a list of
declarations, which carry none. An inline declaration outranks a stylesheet by
ORIGIN, never by out-specifying it.
