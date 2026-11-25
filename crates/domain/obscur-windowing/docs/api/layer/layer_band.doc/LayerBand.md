Which of the four stacking bands a desktop surface sits in.

The bands are ordered, and the order is the whole point: every surface in
[`Background`](LayerBand::Background) sits below every surface in
[`Bottom`](LayerBand::Bottom), and so on up. Ordinary application windows sit
**between `Bottom` and `Top`**, which is what makes a panel in `Top` stay above
them and a wallpaper in `Background` stay below.

Choosing one is a policy decision, not a hint. Where a platform names the same
four bands a backend maps them one to one; where it names fewer, the cut that
must survive is the one between `Bottom` and `Top`, because it is the only one
an application can observe. See [`is_above_windows`](Self::is_above_windows).
