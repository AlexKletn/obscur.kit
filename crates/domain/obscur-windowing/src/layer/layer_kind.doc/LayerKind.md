Which of the compositor's four stacking bands a layer surface sits in.

The bands are ordered, and the order is the whole point: a compositor stacks
every surface in [`Background`](LayerKind::Background) below every surface in
[`Bottom`](LayerKind::Bottom), and so on up. Ordinary application windows sit
**between `Bottom` and `Top`**, which is what makes a panel in `Top` stay
above them and a wallpaper in `Background` stay below.

Choosing one is a policy decision, not a hint: a compositor honours it
exactly, and there is no negotiation.
