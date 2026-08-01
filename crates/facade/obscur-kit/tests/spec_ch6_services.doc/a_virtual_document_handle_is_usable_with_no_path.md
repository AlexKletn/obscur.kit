§6.10 Document access: "File pickers return `FileAccess`/`DocumentHandle`,
not necessarily filesystem paths." and "- optional path hint."

A cloud/virtual document has **no** path hint yet is fully usable — metadata
and bytes both read — while a local file's hint is present and exact.
