How an explicit [`width`](Self::width) / [`height`](Self::height) is
interpreted against the box model (`box-sizing`).

[`BoxSizing::ContentBox`] (the CSS default) treats an explicit size as the
content extent (border box = size + border + padding);
[`BoxSizing::BorderBox`] treats it as the border box (content extent =
size − border − padding, floored at `0`). An `Auto` size is unaffected.
