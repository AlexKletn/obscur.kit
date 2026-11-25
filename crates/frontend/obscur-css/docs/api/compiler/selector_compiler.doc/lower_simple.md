Lowers one parsed simple selector into the `obscur-style` model.

Type/universal/class/id/pseudo-element map directly; attribute and
pseudo-class selectors delegate to [`lower_attr_op`]/[`lower_pseudo_class`].
