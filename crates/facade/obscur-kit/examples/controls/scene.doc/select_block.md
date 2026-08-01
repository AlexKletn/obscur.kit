Builds the select block: a clickable header (fires [`CMD_SEL_TOGGLE`], showing
the selected label or a prompt), the option list rendered **only while the real
[`Select::is_open`] is true** (each option a composed clickable with its own
command), and a disabled header variant.
