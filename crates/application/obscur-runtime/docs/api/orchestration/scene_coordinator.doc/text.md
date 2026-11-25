The phase's text engine: the multi-font paragraph orchestrator, its default
fallback face and the optional hyphenation port, which shape every run this
pass paints AND every box the layout phase measures (see
[`ParagraphShaping`] for why
those three are one entity). The walk itself is otherwise a pure function of
its arguments, which is what keeps composition deterministic and IO-free.
