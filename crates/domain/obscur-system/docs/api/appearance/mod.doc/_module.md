What the desktop says about how things should look, in the kit's own terms.

**Not in the spec.** §5 mentions a theme as a snapshot input and §6 covers
windows and services; neither describes reading the desktop's appearance
settings, so nothing here cites a section number. It is an extension.

The shape follows from one fact: **desktops answer different subsets.** A
GNOME session reached through `gsettings` names its theme and its fonts and
never its colours; a KDE session's `kdeglobals` is an INI with a real palette
in it. So every field is an [`Option`], and absent means *the desktop did not
say* — never a default somebody chose here. A caller that wants a fallback
writes one down where a reader can see it.
