Builds the `zenity` argv for `request`, excluding the tool name (spec §6.9).

This is the pure request→argv mapping, factored out so the flag choices can be
unit-tested without spawning `zenity`:
- [`OpenFile`](DialogRequest::OpenFile) → `--file-selection` (+ `--multiple`).
- [`SaveFile`](DialogRequest::SaveFile) → `--file-selection --save
  --confirm-overwrite --filename <suggested_name>`.
- [`PickFolder`](DialogRequest::PickFolder) → `--file-selection --directory`.
- [`Message`](DialogRequest::Message) → `--info`/`--warning`/`--error --title
  <title> --text <body>`.
- [`Confirm`](DialogRequest::Confirm) → `--question --title <title> --text
  <body>`.
