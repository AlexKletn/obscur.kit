Repository paths whose warnings this profile treats as violations.

Empty for a profile that adds no coverage of its own. Non-empty for one that
compiles code no other build does — where a warning is not noise but the only
report anybody gets.

The workspace carries a warning baseline in the hundreds, so this is deliberately
a short list of paths rather than `-D warnings`: it must catch a field the layer
runner forgot to read without adopting every lint in the graph.
