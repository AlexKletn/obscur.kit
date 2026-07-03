# 4. Layout, CSS and Motion

## 4.1 Layout models

Core layout supports:

- flow/block;
- flex;
- grid;
- positioning;
- intrinsic sizing;
- min/max constraints;
- gap;
- alignment;
- overflow and clipping.

Additional specialized models:

- constraint layout;
- virtualized layout;
- docking workspace.

Excluded legacy/browser baggage:

- float/clear;
- table layout as a general-purpose layout engine;
- print fragmentation;
- legacy compatibility hacks.

Semantic primitives such as `Row`, `Grid` and `Dock` may exist, but there is no requirement for a full Rust-native style DSL.

## 4.2 Measure and arrange

All layout models use a common two-phase contract:

1. `measure`;
2. `arrange`.

A layout context:

- owns child constraints and dependencies;
- forms an invalidation boundary;
- may cache measure/layout results;
- can stop propagation when geometry remains unchanged.

Custom layout uses restricted deterministic domain APIs for geometry and constraints.

## 4.3 Incremental layout

Changes are classified into:

- composition-only;
- paint;
- text;
- style;
- layout;
- tree;
- window/environment.

Recalculation starts at the nearest affected layout context.

Propagation:

- upward only when output geometry changes;
- downward only when constraints change.

Mechanisms:

- containment;
- isolated regions;
- dependency graph;
- generation/version stamps;
- measure/layout caches;
- typed invalidation reasons;
- early exit when geometry is stable.

Scattered ad hoc dirty flags are not the architectural model.

## 4.4 Geometry observers

Supported observers:

- `LayoutObserver`;
- `ResizeObserver`;
- `VisibilityObserver`;
- `IntersectionObserver`;
- optional `TransformObserver`.

Callbacks:

- run after layout commit;
- are coalesced;
- schedule the next transaction;
- cannot recursively mutate the current layout pass;
- are scoped and auto-cleaned.

Animation may access direct before/after geometry snapshots.

## 4.5 Coordinate spaces

Typed spaces:

- Local;
- Parent;
- Window;
- Screen;
- PhysicalPx;
- Normalized.

Rules:

- layout uses logical units;
- renderer converts to physical pixels;
- fractional scale and multiple DPI values are supported;
- viewport and container-relative units are supported;
- normalized coordinates are available for Canvas, gestures and shaders;
- scroll, portals and nested transforms participate in the transform chain.

## 4.6 Subpixel and rounding policy

Layout and transform chains preserve fractional logical geometry until rendering.

Rules:

- no recursive rounding during layout;
- hit testing uses exact fractional geometry;
- pixel snapping happens at paint/render time;
- text, thin lines, borders and icons may use specialized snapping policies;
- flex/grid remainder distribution is deterministic;
- animations are not rounded frame-by-frame.

Possible public controls:

- `pixel-snap: auto`;
- `pixel-snap: none`;
- `pixel-snap: edges`;
- specialized text/vector modes.

## 4.7 Hit testing

Hit testing accounts for:

- layers and z-order;
- transforms;
- clipping;
- scrolling;
- portals;
- custom shapes;
- pointer-event policy;
- Canvas custom hit tests.

An incremental spatial index is used.

The result is a hit chain that maps visual geometry back to the logical event tree.

DevTools uses the same mechanism.

## 4.8 Scroll

Scroll is a base subsystem with:

- offset;
- viewport;
- content extent;
- velocity;
- wheel/touch/trackpad input;
- instant, smooth and kinetic motion;
- chaining;
- overscroll;
- snapping;
- scroll-to and scroll-into-view;
- preserve/restore;
- scroll anchoring;
- DnD/selection autoscroll.

Scroll should remain composition-only where possible.

Viewport information drives virtualization and can feed animations, sticky positioning, hit testing, popups, accessibility and focus.

## 4.9 Virtualization and collections

Virtualized layout supports:

- materialization;
- overscan;
- stable item identity;
- estimated geometry;
- scroll anchoring.

Headless collection infrastructure includes:

- table;
- tree;
- tree-grid;
- rows/columns;
- resizing/reordering;
- sticky headers;
- selection;
- keyboard/spatial navigation;
- accessibility;
- DnD.

These are reusable mechanisms, not a single mandatory heavy widget.

## 4.10 Zoom and pan

Shared zoom/pan infrastructure supports:

- cursor-centered zoom;
- gesture-centered zoom;
- min/max scale;
- fit width;
- fit content;
- panning;
- correct hit testing;
- accessibility integration;
- scroll and gesture integration.

## 4.11 Infinite workspaces

Virtual scenes are separate from ordinary DOM-like layout.

They use:

- world coordinates;
- camera/viewport transform;
- spatial indexing;
- culling;
- virtual materialization;
- level of detail;
- world-space selection;
- snapping/guides/grids;
- origin rebasing;
- visible-region updates.

## 4.12 CSS as the styling language

CSS is the primary and essentially only general styling language.

Rust APIs are limited to:

- CSS custom properties;
- narrow runtime overrides;
- special dynamic values and motion inputs.

CSS parsing is cached per unique source/version and invalidated explicitly.

## 4.13 Supported CSS areas

Supported:

- cascade;
- specificity;
- inheritance;
- custom properties;
- complex selectors;
- attribute selectors;
- `:nth-*`;
- media queries;
- container queries;
- logical properties;
- `calc`, `min`, `max`, `clamp`;
- hover/focus/interaction states;
- flow/flex/grid/positioning;
- typography;
- dimensions and spacing;
- backgrounds;
- opacity;
- transforms;
- overflow/clipping;
- shadows;
- blur;
- filters;
- cursor.

Excluded or restricted:

- float/clear;
- generic table layout;
- columns;
- print/page fragmentation;
- shape-outside;
- browser link/visited/target state;
- vendor hacks;
- arbitrary network loading via `url()` and `@import`;
- legacy clip/zoom/page-break features;
- unnecessary HTML-specific counters/generated-content features.

## 4.14 CSS cascade

Normal origins, low to high:

1. system defaults;
2. theme stylesheet;
3. global application CSS;
4. component/module CSS;
5. inline CSS;
6. runtime override.

Within an origin:

1. specificity;
2. source order.

Module scoping restricts applicability but does not artificially increase specificity.

Scoped themes inherit from their root. Portals preserve the logical parent's style/theme scope.

Custom properties participate in cascade and inheritance before computed-value resolution.

### `!important`

`!important` remains supported in CSS.

- important declarations outrank normal declarations;
- origin/specificity/source-order still resolve conflicts among important declarations;
- runtime override is an explicit runtime layer above authored CSS, including authored `!important`;
- component authors are discouraged from routine use.

## 4.15 Style scopes

- global stylesheets are allowed;
- component stylesheets are scoped by default;
- inherited properties and custom properties cross scope boundaries;
- internal classes do not need global uniqueness;
- components may export style parts and states;
- portals preserve logical style scope;
- explicitly open/global scopes are possible.

CSS module mode:

- applies only when attached to a component/scope root;
- isolates local identifiers;
- keeps normal cascade/inheritance/variables inside the scope;
- is reusable without global side effects.

## 4.16 Generated and synthetic parts

`::before` and `::after` are supported for decoration only.

Meaningful structure or behavior must use real nodes.

Other system-generated parts may use synthetic retained nodes hidden from the component tree but visible in DevTools:

- selection;
- scrollbar parts;
- placeholders;
- exported style parts.

## 4.17 Motion engine

Motion is a separate subsystem, not part of the CSS style engine.

It supports:

- style interpolation;
- transforms;
- layout-affecting animations;
- arbitrary typed interpolation;
- time-driven motion;
- progress-driven motion;
- value-driven motion;
- drag/cursor/gesture/scroll signals;
- interruption;
- reversal;
- chaining;
- synchronization;
- shared-element transitions;
- stable identity;
- before/after layout geometry;
- pre-layout and post-layout phases.

CSS `transition`, `@keyframes` and `animation-*` are parsed as a compatibility/declarative frontend and compiled into native timelines, tracks and interpolators.

Pipeline:

1. style engine resolves base/computed values;
2. motion engine produces presentation values;
3. capability resolution chooses realized values/fallbacks;
4. paint/render consumes realized presentation.

The motion engine remains smooth under event-driven scheduling and skipped frames.
