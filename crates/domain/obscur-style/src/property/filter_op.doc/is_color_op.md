Whether this op is an affine **color** transform (one of the color-matrix
functions the coordinator folds into a single 4×5 matrix) — i.e. anything
that is neither a [`DropShadow`](FilterOp::DropShadow) nor a
[`Blur`](FilterOp::Blur) (both spatial, not color).
