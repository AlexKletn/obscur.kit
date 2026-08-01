A CSS `filter` blurring and/or colour-transforming the node's whole
composited subtree: `(node, border_box, blur_radius (0 = none),
color_matrix)`. The clip carries the element's border box so the renderer
knows the region; this filter layer is the OUTERMOST subtree wrapper, so the
Vello renderer's owned wgpu passes operate on the fully-composited element.
