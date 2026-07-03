# 5. Rendering, Text and Resources

## 5.1 Scene and Paint

Scene/Paint is independent from the concrete backend.

Common vector model:

- paths;
- fills;
- strokes;
- linear/radial/conic gradients;
- clip paths;
- masks;
- borders;
- radii;
- shadows;
- blend modes;
- typed brushes.

The same model is used by:

- CSS decoration;
- SVG;
- Canvas;
- built-in components;
- plugin UI;
- headless output.

## 5.2 Renderer abstraction

The base 2D backend is Vello, hidden behind a renderer abstraction.

Additional backends may include:

- software/headless;
- specialized platform backends;
- WASM/WebGPU;
- test renderers.

Backend determinism boundary:

Must match across backends:

- layout;
- hit testing;
- event behavior;
- paint order;
- clipping;
- scene geometry;
- Scene/Paint snapshots.

Allowed differences:

- rasterization;
- anti-aliasing;
- font rendering details;
- backend-specific precision.

Golden tests use tolerances.

Unsupported effects and fallbacks are explicit diagnostics.

## 5.3 Canvas

Canvas is a built-in custom rendering surface.

It participates in:

- layout;
- clipping;
- transforms;
- hit testing;
- input;
- accessibility where declared;
- composition.

The portable Canvas API uses the common Paint model.

Trusted privileged extensions may use:

- backend-specific GPU resources;
- custom render passes;
- restricted user shaders.

These require capability checks and do not redefine ordinary Canvas semantics.

## 5.4 Overlay host mode

External engines may own:

- window;
- event loop;
- render graph;
- lifecycle;
- frame callback.

The kit receives:

- viewport;
- input;
- clock;
- render target;
- frame scheduling hooks.

It returns render commands, pass descriptions or textures through `RenderHost`/`OverlayHost`.

## 5.5 Foreign surfaces

Optional `ForeignSurface` integrates:

- video;
- WebView;
- camera;
- native controls;
- external GPU surfaces.

A foreign surface may be texture-backed or a native child surface/window.

Restrictions may apply to:

- transforms;
- clipping;
- opacity;
- blur;
- stacking;
- accessibility;
- capture.

Capabilities and limitations are explicit.

## 5.6 Offscreen output

Supported targets:

- texture;
- bitmap/image;
- thumbnail;
- preview;
- node/subtree screenshot;
- optional vector/PDF output where backend capabilities allow.

Inputs may configure:

- viewport;
- DPI;
- theme;
- locale;
- deterministic resources.

Paged document/print rendering is not a core goal.

## 5.7 Color

Baseline:

- sRGB external semantics;
- linear compositing for blending, blur and filters;
- premultiplied alpha;
- image color profiles;
- explicit monitor/window color spaces;
- wide-gamut/HDR capability discovery;
- safe sRGB fallback;
- typed colors.

## 5.8 Clips and masks

Supported:

- rectangular clips;
- rounded clips;
- path clips;
- alpha masks;
- luminance masks.

Rules:

- offscreen passes are created only when required;
- backend caches may optimize masks/clips;
- unsupported combinations use explicit fallback and diagnostics;
- design remains extensible.

## 5.9 Shaders and filters

Restricted user shaders may be used by approved filters.

Requirements:

- validated profile;
- controlled inputs;
- resource limits;
- execution limits;
- no unrestricted memory or platform access;
- capability checks.

Direct backend passes are trusted-only extension APIs.

## 5.10 Text subsystem

Text is an independent domain supporting:

- shaping;
- font fallback;
- Unicode;
- bidi;
- ligatures;
- line breaking;
- caret;
- selection;
- editable text;
- IME;
- rich text spans;
- CSS typography integration.

## 5.11 Editing engine

Infrastructure includes:

- caret;
- single and multi-selection;
- word/line/document navigation;
- Home/End;
- deletion semantics;
- clipboard;
- local undo/redo primitives;
- platform keymap;
- IME.

`TextInput`, `TextArea`, code editors and rich editors are components built on the engine.

Application-wide undo/redo is outside the toolkit.

## 5.12 IME

Typed lifecycle:

- start;
- update;
- commit;
- cancel.

Rules:

- preedit is separate from committed text;
- composition is anchored to a primary caret;
- multi-selection is restricted during composition;
- external edits validate the composition range;
- focus loss follows platform policy;
- one composition is one undo unit;
- candidate-window placement uses caret geometry;
- sensitive/password modes are respected;
- duplicate keydown/text delivery is avoided.

Input hints may include:

- purpose;
- multiline;
- autocorrect;
- capitalization;
- spellcheck;
- sensitive mode;
- preferred language.

## 5.13 Optional language services

`TextServiceProvider` may supply:

- spellcheck;
- grammar;
- autocorrect;
- language detection;
- user dictionary.

Diagnostics are:

- typed;
- ranged;
- asynchronous;
- incremental;
- versioned.

Multiple providers and ignored/sensitive ranges are supported.

The engine is not required in the core package.

## 5.14 Form controls

Minimal value model:

- controlled/uncontrolled;
- current value;
- committed value;
- input event;
- change event;
- disabled;
- readonly;
- invalid.

The toolkit does not provide a mandatory form-state framework.

## 5.15 Resource Hub

A centralized versioned `ResourceHub` manages typed handles for:

- images;
- fonts;
- SVG;
- CSS;
- shaders;
- atlases;
- embedded assets.

Features:

- async load/decode/compile;
- deduplication;
- loading/ready/failed states;
- CPU representations;
- backend/GPU cache;
- versioning;
- stale-job rejection;
- development hot reload;
- custom loaders.

Sources:

- embedded;
- file;
- memory;
- URL through user/host loader;
- generated content.

The kit does not perform arbitrary network access by itself.

## 5.16 Images

Image support:

- bytes;
- stream;
- `DocumentHandle`;
- embedded source;
- generated source;
- raster/vector;
- progressive decode;
- intrinsic size;
- aspect ratio;
- object fit/position;
- animated images;
- DPI variants;
- color profiles;
- safety limits and fallback.

## 5.17 SVG

SVG remains vector through the scene pipeline where possible.

`currentColor` is required and resolves from computed `color`, allowing theme/state changes without reparsing.

## 5.18 Fonts

Default font provider:

- system fonts on native platforms;
- bundled fallback set for WASM/isolated environments;
- fallback resolution;
- shaping;
- glyph caching;
- usable rendering out of the box.

The provider is replaceable.

## 5.19 Caches

Runtime caches may include:

- paint/display-list cache;
- paths;
- glyphs;
- gradients;
- masks;
- textures;
- optional raster cache.

Requirements:

- memory budgets;
- eviction;
- visibility priority;
- correctness transparency;
- DevTools metrics;
- per-cache disable switches for tests.

## 5.20 Persistent derived cache

Optional `CacheStore` persists expensive derived artifacts.

It is intended for startup acceleration, not every runtime access.

Requirements:

- versioned;
- content-addressed where useful;
- compatibility-aware;
- disposable;
- safe to delete.

## 5.21 Startup

Progressive startup targets:

- minimal path to first interactive frame;
- lazy initialization of heavy subsystems;
- critical CSS/font/icon preload;
- no wait for all resources;
- placeholders and fallback first;
- later visual improvement.

Metrics:

- cold start;
- startup phases;
- first window;
- first frame;
- time to interactive.

General hot reload is development-only.

Theme packages may be dynamically replaced in production, transactionally, with last-good fallback.
