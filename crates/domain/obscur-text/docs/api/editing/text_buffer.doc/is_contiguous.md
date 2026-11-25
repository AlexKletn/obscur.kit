Returns whether `edit` is contiguous with the last edit of `unit` for the
purposes of coalescing a `kind` run.

* Typing coalesces only when the new insertion begins exactly where the
  previous insertion ended.
* Deletion coalesces when the new range is adjacent to the previous one
  on either side: the backspace case (new end meets previous start) and
  the forward-delete case (new start equals previous start, since forward
  deletion keeps eating text at the same offset).
