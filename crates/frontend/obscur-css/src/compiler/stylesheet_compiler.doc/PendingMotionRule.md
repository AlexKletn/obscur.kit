A style rule's motion before `@keyframes` names are resolved to tracks: the
rule's selector list, its transition templates, and its (still-by-name)
animation references. Resolved into a [`CompiledMotionRule`] in
[`StylesheetCompiler::compile`], where every `@keyframes` rule is known.
