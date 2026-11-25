Returns a MIME type for a file `extension`, or `None` when it is not in this
minimal map.

This is a deliberately small, best-effort table covering the common UI asset
extensions this adapter is expected to serve (images, vector art, stylesheets,
JSON, plain text, and web fonts). It is not a comprehensive MIME registry;
callers that need authoritative typing should sniff the bytes. The lookup is
case-insensitive.
