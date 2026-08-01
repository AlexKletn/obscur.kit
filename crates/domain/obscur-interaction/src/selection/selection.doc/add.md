Adds `node` to the selection and moves the lead onto it.

The existing anchor endpoint is preserved (so a subsequent range still
extends from it); if there was no anchor, one is collapsed onto `node`.
