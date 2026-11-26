Applies the surface transform to a freshly composed display list. The
transform places logical content in physical pixels and accounts for the
content offset and camera. The optional rounded clip encloses this transform
so that its corners stay fixed to the window during content motion.

When neither a transform nor a clip is required, the list is returned
unchanged. Otherwise the wrapper moves the owned paint commands and balances
each opening command with its closing command.
