Appends a synthetic DevTools node for the placeholder part of every field
currently showing one (§4.16). Like the scrollbar parts these are **not**
retained-tree nodes and appear in no node's `children()`; they exist only in
this projection, carrying the field's box and named `placeholder`.
