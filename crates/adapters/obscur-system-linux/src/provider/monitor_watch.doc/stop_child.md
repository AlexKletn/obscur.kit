Kill the tool, which closes the pipe and ends the reader.

The thread is not joined. Joining would make dropping a guard wait on a process
that may be slow to die, and by then there is nothing left for the thread to do.
