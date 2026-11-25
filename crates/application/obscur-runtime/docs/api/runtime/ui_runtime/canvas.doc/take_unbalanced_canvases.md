Takes the painters that left their region unbalanced since the last call.

A painter that opens a clip and never closes it would swallow everything painted
after it; the runtime closes it and names the painter here rather than repairing
it in silence. Taking rather than reading means a host reports each fault once.
