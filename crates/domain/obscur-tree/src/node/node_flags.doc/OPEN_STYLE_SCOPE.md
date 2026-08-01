Set alongside [`COMPONENT_HOST`](Self::COMPONENT_HOST) when the mounted
component declared an **open** style scope (spec §4.15 "explicitly
open/global scopes are possible").

Style scoping is decided by walking a node's ancestors for the nearest
component host, and an open host is transparent to that walk: the
elements it rendered are matched by the document's ordinary selectors
exactly as authored elements are. Meaningless on a node that is not a
component host; the runtime sets it from the contract at mount and
clears it at unmount, together with `COMPONENT_HOST`.
