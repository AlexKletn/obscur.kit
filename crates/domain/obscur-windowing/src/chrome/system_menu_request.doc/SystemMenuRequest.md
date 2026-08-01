A request to pop the platform's system (window) menu at a point (spec §6.4).

The position is in window-local logical pixels — typically the cursor
location of a right-click on a custom title bar. On platforms without a
system menu this request is a no-op that the backend reports via a
`WindowOperationResult`; that reporting is not modeled here.
