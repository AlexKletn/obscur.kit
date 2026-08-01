Taken by the loop when the surface is opened. `Arc<Mutex<…>>` rather than
a plain field because a waker is cloned before the loop exists and only
one clone can hand the source over.
