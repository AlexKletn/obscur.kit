The origin of a drag, carrying the data being moved (spec §6.21 typed
source).

A drag source names the [`origin`](DragSource::origin) node the drag started
from, the [`payload`](DragSource::payload) it carries — which may expose both
an internal typed value and system MIME (spec §6.21) — the
[effects](DragEffect) the source permits, and an optional
[`preview`](DragSource::preview) image descriptor. A [`DragSession`] begins
from a source and negotiates its effect against each drop target.

[`DragSession`]: super::DragSession
