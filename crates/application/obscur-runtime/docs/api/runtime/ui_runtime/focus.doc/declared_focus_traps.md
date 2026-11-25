The trapping scopes for the given declaring nodes.

The scope's id is the declaring node's, which is what lets it be recognised again
next frame without a side table, and its members are that node and every node
beneath it.

Membership is the whole subtree rather than only its tab stops: the scope is
§2.6's command-routing link as well as the focus ring's boundary, and a command
raised on a non-focusable node inside a dialog is still a command inside that
dialog.
