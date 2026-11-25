Maps `role` to the closest AccessKit [`Role`].

Most roles map one-to-one. The approximations are:
- [`SemanticRole::Text`] → [`Role::Label`] (AccessKit's static-text
  role);
- [`SemanticRole::ProgressBar`] → [`Role::ProgressIndicator`];
- [`SemanticRole::Separator`] → [`Role::Splitter`] (AccessKit has no
  distinct separator role);
- [`SemanticRole::Unknown`] → [`Role::Unknown`].
