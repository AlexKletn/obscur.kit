How a [`UiRuntime`] starts: the one constructor, and the initial value of every
field it seeds.

It is here rather than beside the struct because the struct's file states *what*
the runtime holds and this states *what each of those things begins as* — two
different readings of the same 47 fields, and the second one is the only place
the "empty tree, idle scheduler, no interaction recorded, first frame at
[`FrameId::FIRST`]" contract is written down. The two decisions that are not
mechanical say so where they are made: the locale starts at `und` rather than
`en` (§7.2), and the diagnostics sink starts null rather than absent so every
report path exists before a host installs anything.

[`FrameId::FIRST`]: crate::frame::FrameId::FIRST
