The item's total border + padding along the given axis (`horizontal` for the
row main / column cross axis, else vertical).

Used to expand an explicit `content-box` size into the border-box extent the
item actually occupies, so a `width: 0` / `height: 0` item with a border keeps
its border widths instead of collapsing to a zero-extent box.
