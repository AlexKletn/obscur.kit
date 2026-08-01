The design metrics standing in for a face whose own tables are unreadable:
the idealized 0.8 / 0.2 ratios expressed at 1000 units per em, so a scaled
line height equals the font size exactly. A face resolved with these is
reported as metric-less by [`FontRegistry::face_metrics`], so the shaper's
fallback to the same ratios is observable rather than a silent pretence of
real data.
