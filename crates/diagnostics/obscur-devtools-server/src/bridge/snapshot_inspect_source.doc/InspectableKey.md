A hashable/orderable key for [`Inspectable`], which is `Copy` + `Eq` but not
`Ord`; keying a [`BTreeMap`] on its stable discriminant avoids requiring the
protocol enum to derive `Ord`.
