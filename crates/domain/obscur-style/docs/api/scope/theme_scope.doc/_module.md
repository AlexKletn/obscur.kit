A scoped theme applied to a subtree.

Per §4.14/§4.15, a theme can be scoped to a subtree rooted at a style scope.
A scoped theme inherits from its root / parent theme by default, so it
*supplements* the ambient theme (adding or overriding tokens) rather than
replacing it; a non-inheriting scoped theme instead starts fresh from the
root theme. This type models that identity plus the inherit-from-parent flag;
resolving the actual theme tokens lives elsewhere.
