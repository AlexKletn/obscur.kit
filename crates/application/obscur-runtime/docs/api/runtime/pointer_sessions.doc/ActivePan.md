A pan armed for one pointer: the recognizer deciding whether the contact is a
drag, and the scroll container it would drag.

The container is resolved **once**, where the contact went down, and never
re-resolved: a drag belongs to the list it started in, even once the finger
has left that list's bounds (which is exactly what the pointer capture taken
on the first delta guarantees).
