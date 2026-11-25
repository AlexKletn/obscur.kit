Builds the `notify-send` argv for `notification`, excluding the tool name
(spec §6.12).

This is the pure notification→argv mapping, factored out so the flag choices
can be unit-tested without spawning `notify-send`. When the notification
[requests attention](Notification::request_attention) the argv leads with
`--urgency=critical`; the title and body then follow as positional arguments.
