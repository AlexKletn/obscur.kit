# 2. Components, Tree and DSL

## 2.1 Component model

A component is a scoped View unit with:

- creation;
- declarative view output;
- typed props;
- optional local view state;
- scoped subscriptions;
- scoped async tasks;
- minimal lifecycle hooks;
- automatic cleanup;
- default error-boundary behavior.

Components do not own application or domain state frameworks.

## 2.2 Tree model

External model:

- components produce a desired declarative tree;
- child identity is stable through explicit keys;
- updates are expressed as new desired state rather than direct retained-tree mutation.

Internal model:

- reconciliation updates a stable retained tree;
- nodes keep stable IDs where identity is preserved;
- layout, focus, selection, animation, DnD and caches attach to retained identity.

An internal ECS representation may be introduced as an optimization later, but it must not leak into the API.

## 2.3 Events

Event propagation:

1. capture;
2. target;
3. bubble;
4. default action.

Event API includes:

- `target`;
- `current_target`;
- `stop_propagation`;
- `stop_immediate_propagation`;
- `prevent_default`;
- trusted/untrusted origin.

Events are typed. String-based method dispatch is not used.

## 2.4 Pointer, gesture and hit behavior

The pointer model supports mouse, touch, pen and trackpad-like sources:

- pointer ID;
- device type;
- buttons;
- logical position and delta;
- pressure;
- tilt;
- rotation;
- primary-pointer status;
- hover capability.

Pointer capture is per pointer ID and is released on pointer completion, node removal, window close or system loss.

Competing recognizers use a gesture arena.

Touch does not synthesize persistent hover. Pen hover depends on device capability.

## 2.5 Focus

Focus is a first-class subsystem:

- focus tree;
- scopes;
- tab order;
- programmatic focus;
- focus-visible semantics;
- restore focus;
- modal focus traps;
- interaction with accessibility and pointer input.

Focus is not modeled as a CSS flag. CSS consumes focus state produced by the focus subsystem.

## 2.6 Command model

Input layers:

- physical key;
- logical key/text;
- shortcut;
- semantic command.

Platform adapters map platform key conventions to semantic commands.

Command routing:

1. focused node;
2. focus scope;
3. ancestors;
4. window;
5. application fallback.

IME and text input are separate from keydown.

## 2.7 General selection

A reusable non-text selection model supports:

- single selection;
- multi-selection;
- range selection;
- anchor and active item;
- Ctrl/Cmd/Shift semantics;
- keyboard and spatial navigation;
- DnD integration;
- accessibility;
- virtualization.

Selection policy and domain meaning remain owned by the component or application.

## 2.8 Rust DSL

The primary Rust UI syntax is a proc-macro-based, TSX-like DSL.

Goals:

- typed props and events;
- compact declarative syntax;
- compile-time validation;
- stable source mapping;
- no exposed internal tree builders.

A compose-like ordinary Rust API may exist as a fallback and code-generation target, but is not the preferred verbose public surface.

## 2.9 External `.okui` files

`.okui` is a declarative UI frontend compiled into the same UI IR as the Rust DSL.

Characteristics:

- template-like structure comparable to Angular templates;
- no embedded general-purpose scripting language;
- typed props, events and commands;
- compile-time contract validation;
- no business logic.

Planned IDE support uses plugins and/or LSP.

## 2.10 Expressions

Allowed:

- props;
- read-only state;
- loop/template locals;
- field access;
- comparisons;
- logical operators;
- Option/null checks;
- formatting;
- translations;
- explicitly exported pure functions.

Forbidden:

- assignment;
- arbitrary mutation;
- I/O;
- async work;
- unconstrained arbitrary function calls;
- direct service access.

## 2.11 Conditions and collections

Conditions:

- `If`;
- `ElseIf`;
- `Else`;
- simple conditional attributes.

Collections:

- `For`;
- `VirtualFor`;
- mandatory stable `key`;
- optional empty/loading/separator slots.

These constructs do not create real retained nodes unless their rendered output requires them.

## 2.12 Templates and slots

Templates are stateless and lifecycle-free.

Components may declare:

- named slots;
- allowed child contracts;
- slot cardinality;
- exported style parts;
- exported visual states.

## 2.13 Component contracts

A component exports a typed contract:

- props;
- defaults;
- events;
- slots;
- allowed children;
- style parts/states;
- required capabilities;
- diagnostics metadata.

Unknown attributes, events and string method names are compile errors.

## 2.14 Component registry

`ComponentRegistry` stores:

- stable component ID;
- contract schema;
- version;
- capability requirements;
- factory;
- diagnostics metadata.

Static registration is the default. Dynamic registration is supported by the plugin system.

## 2.15 UI IR

UI IR is:

- typed;
- versioned;
- validated;
- source-mapped;
- independent of direct Rust type names;
- based on stable component/resource/style IDs;
- embedded at build time or loaded from validated plugin packages.

UI IR versioning is independent from the kit version.

Compatibility includes:

- supported version ranges;
- migrations where practical;
- capability negotiation;
- precise diagnostics for newer unsupported IR.

UI IR is an internal/extension contract, not the ordinary application programming model.
