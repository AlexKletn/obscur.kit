# 8. Plugin System

## 8.1 Purpose

The primary plugin use case is adding UI modules to already-built applications:

- panel widgets;
- dashboard widgets;
- screens;
- settings pages;
- workspace tabs;
- context-menu items;
- routes;
- commands.

The architecture also permits more powerful extensions where explicitly allowed.

Plugins are UI-first, not limited to UI-only.

## 8.2 Plugin classes

Two principal classes:

### Trusted native plugin

- separately compiled dynamic library;
- runs in the application process;
- receives an extended but capability-scoped API;
- is trusted code;
- cannot be reliably sandboxed from process memory.

### Sandboxed plugin

Runs through:

- WASM;
- separate local process;
- optional remote service adapter.

Interaction is message-based and capability-scoped.

## 8.3 Basic integration boundary

Plugins integrate through a stable Plugin API.

Capabilities may include:

- register screens/routes;
- register components;
- provide UI IR;
- register actions/commands;
- provide resources/CSS/themes/localization;
- lifecycle hooks;
- request host services.

Plugins never receive direct access to:

- retained-tree internals;
- renderer internals;
- arbitrary application memory;
- raw host platform handles unless a privileged contract explicitly allows it.

## 8.4 Native ABI

Rust ABI is not used as the compatibility boundary.

Native plugin ABI:

- stable C ABI;
- opaque handles;
- versioned function tables;
- `HostApi` and `PluginApi`;
- typed status/error values;
- strict ownership rules;
- no panic across ABI;
- pre-activation compatibility/capability negotiation.

A Rust SDK hides `unsafe` and generates glue.

## 8.5 Package and manifest

A versioned plugin manifest includes:

- plugin ID;
- name;
- version;
- author;
- plugin type: native/WASM/process;
- Plugin API range;
- UI IR range;
- entrypoint;
- routes/screens/components/actions;
- resources;
- CSS;
- themes;
- localization;
- requested capabilities;
- supported platforms and architectures;
- dependencies;
- content hashes;
- optional future publisher signature metadata.

Package containers:

- ZIP;
- TAR;
- `tar.gz`;
- `tar.zst`;
- other future containers through `PackageReader`.

The logical package layout is container-independent.

## 8.6 Package validation

Before code execution:

- validate manifest;
- validate version ranges;
- validate requested capabilities;
- resolve dependencies;
- normalize paths;
- reject `..` and absolute paths;
- reject package-root escape;
- reject symlink/hardlink by default;
- enforce file count and size limits;
- prevent archive bombs;
- verify hashes;
- choose entrypoint only from manifest.

Native libraries are extracted to an isolated host-managed directory with controlled permissions and cleanup.

A partially validated package is never activated.

## 8.7 Optional signatures

Cryptographic package signatures and publisher trust are future/optional layers.

Core requirements that remain mandatory:

- manifest validation;
- hashes;
- compatibility checks;
- capability checks;
- transactional install/update/disable/uninstall;
- rollback on failure.

## 8.8 Capability model

Examples:

- UI/routes/actions;
- own resources/storage;
- file/document access;
- network access;
- clipboard;
- notifications;
- media;
- shell integration;
- inter-plugin services;
- diagnostics/telemetry;
- native/system integration.

Permission classes:

- required;
- optional;
- runtime-requested;
- administratively denied.

User-visible descriptions are understandable and separate from internal capability names.

## 8.9 Security principal

Each `plugin ID + publisher/installation identity` is a security principal.

Permissions, storage, handles and audit attach to that principal.

- version updates preserve identity;
- owner/ID change creates a new principal;
- contributions and instances inherit narrower scopes;
- service calls always carry caller identity;
- sandboxed runtimes cannot impersonate other plugins.

## 8.10 Host policy

The host defines:

- allowed plugin types;
- allow/deny rules;
- available capabilities;
- resource budgets;
- extension points;
- eager/background activation;
- window rules;
- shortcut rules;
- network/file/system restrictions;
- per-contribution policy.

Host policy overrides manifest requests.

## 8.11 Lifecycle

Lifecycle states:

- discovered;
- validated;
- resolved;
- loaded;
- initialized;
- active;
- suspended;
- deactivated;
- unloaded.

Rules:

- no code before validation;
- `initialize` receives only authorized services;
- registrations are transactional;
- failed activation rolls back;
- unload cleans tasks, subscriptions, timers, handles and UI;
- suspension may preserve package/runtime state;
- updates may use side-by-side activation.

## 8.12 Teardown

Shutdown order:

1. stop accepting new commands;
2. move UI to closing/fallback;
3. cancel scoped tasks/streams;
4. revoke capabilities/service handles;
5. remove routes/actions/windows/contributions;
6. save last committed state;
7. release resources/storage transactions;
8. stop executor;
9. logically deactivate native library.

Each phase has a deadline and diagnostics.

A stuck plugin cannot block UI or process shutdown indefinitely.

## 8.13 Physical native unload

Logical deactivation is guaranteed.

Physical `dlclose`/`FreeLibrary` is optional and allowed only when the plugin supports unload and the host verifies no:

- live plugin threads;
- callbacks;
- ABI handles;
- queued plugin code/objects;
- TLS/static resources;
- plugin code on stack.

Otherwise the library remains loaded until process termination.

## 8.14 Extension points

The application publishes typed points and defines their identifiers and contracts.
Examples of application-defined identifiers include:

- `panel.left`;
- `panel.status`;
- `sidebar.tools`;
- `settings.pages`;
- `workspace.tabs`;
- `context-menu.file`;
- `dashboard.widgets`;
- `routes`.

Each point declares:

- allowed UI/contribution type;
- props/context;
- available actions;
- sizing/lifecycle rules;
- instance limits;
- order/grouping;
- required capabilities.

Plugins register typed contributions rather than arbitrary retained-tree injection.

Core contribution types:

- `WidgetContribution`;
- `ScreenContribution`;
- `RouteContribution`;
- `MenuContribution`.

Applications can define their own contribution contracts over the kit’s
registration and UI integration mechanisms.

## 8.15 Non-UI contributions

Optional:

- `BackgroundServiceContribution`;
- `FileHandlerContribution`;
- `ImportExportContribution`;
- `CommandContribution`;
- `ProtocolHandlerContribution`;
- `DataProviderContribution`.

These require stricter activation, permissions and budgets.

Hidden auto-start is not the default.

## 8.16 Nested extension points

A plugin may publish its own typed extension points.

Rules:

- globally namespaced ID;
- explicit contract;
- explicit dependencies;
- host can deny;
- no unadvertised injection into plugin internals;
- disabling the parent removes nested contributions;
- cycles and unbounded nesting are rejected.

## 8.17 Contribution definition and instance

A contribution definition describes an available type.

The host creates instances.

Each instance has:

- stable ID;
- settings;
- local state;
- lifecycle;
- cancellation scope;
- placement;
- size;
- independent restart/removal.

Multiple instances of one definition are allowed.

Host owns placement. Plugin owns internal UI and logic.

## 8.18 Contribution context

A contribution receives a typed `ContributionContext` containing:

- read-only data/signals;
- authorized actions/commands;
- locale;
- theme;
- accessibility/environment;
- scoped storage;
- resources;
- capability handles;
- lifecycle/cancellation.

It never receives direct host state mutation access.

Commands/events return to the host for validation and policy.

## 8.19 UI integration

Primary model:

- plugin provides UI IR and component registrations;
- host constructs a normal retained subtree;
- common layout, CSS, input, accessibility and renderer apply;
- sandboxed logic communicates through messages/signals/commands;
- native component factories still use common runtime contracts.

A remote/texture surface is a special privileged contribution with explicit accessibility, focus, clipping and composition limits.

## 8.20 Theme integration

Plugin subtree inherits:

- host `ThemeProvider`;
- CSS variables;
- locale;
- text direction;
- reduced-motion preference;
- accessibility preferences.

Plugin CSS is scoped by default.

A plugin may:

- define local tokens;
- export style parts;
- provide supplemental theme assets.

The host may override exported parts.

Global plugin CSS requires a capability.

A plugin-specific theme may supplement but not replace the host theme.

Theme changes apply to plugin UI in the same transaction as host UI.

## 8.21 Resources and localization

Plugin resources use namespaced stable IDs.

Resource Hub handles:

- images;
- SVG;
- fonts;
- CSS;
- localization;
- UI IR;
- shaders where permitted.

A plugin cannot access another plugin's resources unless explicitly shared.

Host may overlay selected assets or localization entries.

Plugin update/uninstall invalidates its resources atomically.

## 8.22 Internal plugin runtimes

A plugin may include its own:

- runtime;
- VM;
- event loop;
- interpreter;
- execution framework.

Obscur Kit treats this as an implementation detail.

Only the external contract matters:

- Plugin API;
- UI IR;
- contributions;
- services;
- capabilities;
- lifecycle;
- resource ownership;
- cancellation;
- threading rules.

## 8.23 Plugin executor

Sandboxed execution is abstracted by `PluginExecutor`.

Implementations:

- WASM runtime;
- local process;
- remote service adapter.

Properties:

- message-based;
- no shared pointers;
- versioned serializable DTO;
- async request/response;
- event streams;
- cancellation;
- deadlines;
- backpressure;
- restartable executor;
- host-owned fallback during unavailability.

Specific WASM engines and IPC protocols are not public Plugin API.

## 8.24 Threading and reentrancy

- UI mutations are marshalled through UI transactions;
- Plugin API is thread-neutral except explicitly affinity-bound methods;
- callbacks for one plugin are serialized by default;
- synchronous cyclic calls `host → plugin → host` are forbidden;
- nested calls become async messages/commands;
- native plugins may create threads but cannot mutate UI directly;
- long calls never hold UI transactions or ABI locks;
- platform-handle affinity is enforced by host.

## 8.25 Message semantics

Message categories:

- `query`;
- `command`;
- `event`.

Metadata:

- correlation ID;
- generation;
- operation ID;
- deadline;
- cancellation.

Delivery:

- at-most-once by default;
- retries only for explicitly idempotent operations;
- no exactly-once guarantee;
- explicit ordered channels for ordering;
- duplicate/stale detection;
- restart requires resync or result verification for uncertain commands.

## 8.26 DTO and IDL

One versioned schema/IDL defines:

- Plugin API;
- services;
- events;
- contribution contracts.

Features:

- stable field IDs;
- optional fields;
- tagged unions;
- capability markers;
- generated Rust bindings;
- generated C ABI glue;
- transport serialization;
- no internal Rust types;
- same semantics across native/WASM/process.

Unknown optional fields are ignored. Unknown required variants produce compatibility errors.

## 8.27 Host API services

The Host API is modular:

- `UiService`;
- `ActionService`;
- `NavigationService`;
- `ResourceService`;
- `StorageService`;
- `SecureStorageService`;
- `DocumentService`;
- `NetworkService`;
- `ClipboardService`;
- `NotificationService`;
- `DiagnosticsService`;
- application-specific services.

Each service:

- versions independently;
- may be absent;
- returns `unsupported` when unavailable;
- is issued only through authorized handles.

## 8.28 Handles

Handles are:

- opaque;
- versioned;
- bound to plugin principal and capability scope;
- cloneable only when allowed;
- limited by plugin/contribution/operation lifetime;
- invalidated on revoke/unload/restart;
- leak-tracked by host;
- optionally lease-based for long life.

A plugin cannot hand its handle directly to another plugin.

Cross-plugin access requires a new host-issued scoped handle.

## 8.29 Capability delegation

Only the host delegates capabilities.

Rules:

- derived scope cannot exceed source scope;
- may have expiry;
- may have operation limits;
- may bind to an instance;
- source revocation revokes descendants;
- delegation chain is audited;
- important rights may require user confirmation.

## 8.30 Services between plugins

Plugins may register versioned services.

Consumers declare dependencies.

Host provides capability-scoped handles.

Properties:

- typed async calls;
- required or optional dependencies;
- provider unload invalidates handles;
- host may proxy between native/WASM/process;
- dependency cycles are detected.

Plugins may build their own IPC if needed; that transport is outside the kit so long as host integration still follows policy.

## 8.31 Dependencies

Manifest dependency form:

- plugin ID;
- version range;
- type: required / optional / service-only.

Host builds and validates a dependency graph before activation.

- activation follows dependency order;
- conflicts produce precise diagnostics;
- side-by-side versions require explicit support;
- optional absence degrades only related functionality;
- plugins do not silently install dependencies;
- provider updates are checked against consumers.

## 8.32 Versioning

Version independently:

- Plugin API;
- UI IR;
- contribution contracts;
- service contracts;
- resource manifest;
- plugin state schema.

Plugins declare supported ranges.

Compatible extension uses capabilities and additional function-table entries.

Breaking changes require a major version.

## 8.33 Contribution identity and conflicts

Global ID form:

`plugin-id/local-id`

Rules:

- duplicate ID inside one plugin fails activation;
- plugins cannot silently replace each other's contributions;
- route/action/service conflicts are resolved by host policy;
- priority and ordering constraints are supported;
- fallback contributions are allowed;
- overriding an existing extension requires permission;
- all registrations belong to one plugin version and are removed atomically.

## 8.34 Partial activation

Each contribution is checked independently for:

- contract compatibility;
- capabilities;
- dependencies;
- extension-point policy.

Statuses:

- available;
- degraded;
- blocked;
- failed.

Failure of one contribution does not disable unrelated contributions.

A contribution may be revalidated and reactivated when conditions change.

## 8.35 Lazy activation

Manifests and contribution metadata are indexed without executing code.

Default behavior for panel/dashboard UI:

- cold until first use;
- lazy activation;
- host-owned loading placeholder;
- suspend when hidden;
- unload after inactivity by policy;
- restore instance state on reactivation.

Eager activation is limited to declared reasons such as:

- background service;
- startup action;
- file handler.

Host approval is required.

## 8.36 Contribution sizing

Each contribution declares:

- minimum size;
- preferred size;
- maximum size;
- aspect ratio;
- orientation;
- resize support;
- presentation modes: compact / regular / expanded.

Host supplies actual constraints and owns final size.

Contributions respond to container queries and provide compact/fallback UI for constrained areas.

## 8.37 Visibility and activity

Host communicates:

- mounted;
- visible;
- occluded;
- focused;
- interactive;
- background;
- suspended.

Invisible contributions may stop layout/paint.

Animations and high-frequency updates may pause.

Background work requires policy.

Resume uses fresh snapshot/resync.

## 8.38 Host-owned chrome

Host may own:

- title;
- icon placement;
- drag handle;
- resize controls;
- menu;
- close action;
- settings action;
- loading/error/suspended/permission UI.

Plugin supplies content and metadata.

Extension points may explicitly allow `chrome: none`.

## 8.39 Error boundaries

Every contribution instance has a mandatory host-owned outer error boundary.

Properties:

- failure is localized to that widget/screen;
- host renders fallback;
- restart can target one contribution;
- diagnostics include plugin ID and instance ID;
- plugin internal component boundaries continue below it;
- plugin cannot disable the host boundary.

## 8.40 Settings

Two levels:

- plugin-global settings;
- contribution-instance settings.

Plugin declares a versioned typed schema:

- types;
- defaults;
- validation;
- enums/ranges;
- secret/resource/document fields;
- field dependencies;
- scope;
- migrations.

Host may auto-generate settings UI.

An application can publish a view integration point for plugin-owned settings UI.
Settings changes in that application can follow:

`draft → validate → apply → commit/cancel`

Secrets use secure storage.

## 8.41 Storage

Each plugin has an isolated namespace.

Separate areas:

- global state;
- instance state;
- cache;
- secrets.

Properties:

- host-defined quotas;
- transactional write;
- versioning;
- migrations;
- backup;
- cache clear without data loss;
- secrets only via `SecureStorageService`.

One logical contract applies to native/WASM/process.

## 8.42 State categories

Contribution state is split into:

- ephemeral view state;
- restorable UI state;
- plugin domain state;
- host placement state.

Ephemeral state is not migrated.

Restorable UI state may be persisted by host.

Domain state belongs to the plugin storage schema.

Placement belongs to host layout/workspace.

## 8.43 State synchronization

Host sends:

- initial versioned snapshot;
- typed patches/signals;
- generation/sequence IDs.

Rules:

- stale messages are dropped;
- updates are coalesced;
- desync triggers full resync;
- subtree changes commit atomically;
- host state is read-only;
- plugin commands pass host validation;
- plugin view state remains plugin-owned.

## 8.44 Migration and update

Plugin storage is namespaced and versioned.

Migration:

- runs before activation;
- is explicit;
- uses declared state-schema ranges;
- can be backed up;
- rolls back to old version on failure.

Side-by-side versions do not share mutable state directly.

Uninstall data retention is host policy.

## 8.45 Staged handover

Update procedure:

1. load/validate new version beside old;
2. migrate state;
3. trial activation;
4. instantiate new contributions;
5. atomically switch UI;
6. require successful first commit;
7. deactivate old version.

On failure, host restores old instances.

No empty panels or lost routes should appear during handover.

## 8.46 Orphaned instances

On disable/uninstall:

- contributions are removed atomically;
- placement/settings may remain;
- host stores a placeholder with plugin and contribution IDs;
- compatible reinstall restores the instance;
- incompatible schema requires migration/reset;
- user may delete orphaned data;
- host does not silently collapse layout.

## 8.47 Windows and popups

Plugins request windows through host services.

Host decides:

- window;
- tab;
- modal;
- popup;
- denial.

Host owns the native window.

Capabilities control fullscreen, transparency, always-on-top and custom chrome.

Sandboxed plugins never receive raw native handles.

Popups use common layer/portal infrastructure.

## 8.48 Navigation

Route contributions are namespaced.

Host controls insertion into:

- root;
- settings;
- workspace;
- modal flow.

Rules:

- no route hijacking without permission;
- deep links go through host router and policy;
- route state is versioned DTO;
- history remains host-owned;
- unload replaces/closes active plugin routes safely;
- navigation requests use `NavigationService`.

## 8.49 Actions and shortcuts

Plugins register actions through `ActionService`.

- shortcuts use host command routing;
- global shortcuts require capability;
- conflicts use host policy;
- system/protected commands cannot be intercepted;
- enabled/checked/visible are typed state;
- unload removes all bindings atomically;
- host may remap/disable shortcuts.

## 8.50 Focus, input and accessibility

Plugin UI participates in the common:

- focus tree;
- Tab order;
- spatial navigation;
- screen-reader tree;
- gesture arena;
- pointer capture;
- accessibility action routing.

Sandboxed plugins receive normalized input, not raw platform handles.

On suspend/unload, focus returns to host.

Modal contributions follow host focus trap and restore policy.

## 8.51 Clipboard and DnD

Plugin UI uses common Data Transfer infrastructure.

Allowed boundary values:

- typed DTO;
- MIME representation;
- `DocumentHandle`.

Not allowed:

- direct internal Rust objects;
- raw system clipboard handles for sandboxed plugins.

Host filters formats by capability.

Lazy providers use async handles.

Unload/cancel invalidates offers/providers.

## 8.52 File access

Sandboxed plugins use:

- `DocumentService`;
- `DocumentHandle`;
- `DirectoryHandle`.

Permissions may include:

- read;
- write;
- create;
- watch.

Persistent access requires a permission token.

A document may be local, portal-backed, cloud-backed or virtual.

Trusted native/process plugins may receive path access only through a separate capability.

## 8.53 Network access

Sandboxed plugins use `NetworkService`.

Possible policy:

- origin allowlist;
- method allowlist;
- request size/time/parallelism limits;
- cancellation;
- streaming;
- scoped credential handles;
- host proxy/TLS/certificate policy;
- per-plugin diagnostics.

Trusted native/process plugins may use their own stack only when host policy allows.

The toolkit defines the service contract, not a general HTTP framework.

## 8.54 Secrets

Secrets are not passed in clear text by default.

Use:

- `SecureStorageService`;
- scoped credential handles;
- origin/service/operation/time restrictions;
- host-owned secret entry UI;
- redacted diagnostics;
- revocable leases.

Raw secret access is reserved for trusted plugins with a high-privilege capability.

## 8.55 Event provenance

Every call/event includes origin:

- host;
- user;
- plugin;
- system;
- synthetic.

Also include:

- plugin principal;
- contribution instance;
- correlation ID.

Plugin-generated input is not trusted user input.

Sensitive default actions require real trusted activation.

Provenance cannot be forged and is visible to diagnostics/audit.

## 8.56 User activation token

`UserActivationToken`:

- comes only from real trusted input;
- binds to principal, contribution and action type;
- has short lifetime;
- is usually one-shot;
- is not freely serializable;
- may cross limited async chains;
- cannot be created by background work or synthetic events.

It may gate:

- file picker;
- clipboard;
- fullscreen;
- notification permission;
- similar user-gesture-sensitive actions.

Host may still require confirmation.

## 8.57 Permission requests

Plugin requests permission through host service.

Host-owned UI displays:

- purpose;
- scope;
- duration;
- reason.

Outcomes:

- granted;
- denied;
- dismissed;
- unsupported.

Grant duration may be:

- one use;
- session;
- persistent.

Background plugins cannot unexpectedly open permission UI without user activation.

Repeated denied requests are throttled.

## 8.58 Capability discovery and revocation

Before activation, the plugin receives an immutable snapshot of available capabilities and service versions.

Capabilities report:

- native;
- emulated;
- limited;
- unavailable.

Plugins query capabilities, not platform names.

Runtime changes trigger typed events and revalidation only for affected contributions.

Host may revoke capabilities dynamically.

Revocation:

- invalidates handles;
- cancels active operations;
- stops related streams/tasks;
- moves contributions to degraded/blocked;
- may later permit reactivation.

## 8.59 Resource budgets and fault isolation

Sandboxed/process plugins may be limited by:

- CPU;
- memory;
- task count;
- subscription count;
- handle count;
- message size;
- call frequency.

Host provides:

- watchdog;
- timeout;
- backpressure;
- suspend/kill;
- diagnostics;
- restart policy.

Trusted native plugins have less isolation.

## 8.60 UI performance budgets

Per contribution:

- UI transaction budget;
- invalidation budget;
- signal/event frequency limit;
- coalescing;
- subtree complexity limit;
- layout/paint/render telemetry;
- throttling;
- suspend;
- lazy activation;
- virtualization requirement for large collections.

Budgets may differ for widgets, full screens and background contributions.

## 8.61 Crash recovery

Host distinguishes:

- normal unload;
- panic/crash;
- timeout;
- quota kill.

Process/WASM executors may restart.

Instances restore last committed state and resync.

Repeated crashes trigger backoff and eventual block.

Host-owned fallback provides restart affordance.

Native in-process crashes are explicitly not guaranteed to be isolated.

## 8.62 Diagnostics and testing

Each plugin ID/version has a diagnostic scope.

DevTools may inspect:

- contributions;
- capabilities;
- services;
- lifecycle;
- executor state;
- resource failures;
- slow calls;
- quota events.

Headless test host supports:

- package loading;
- mock Host API;
- test extension points;
- manifest validation tests;
- ABI/API negotiation tests;
- migration tests;
- unload-cleanup tests;
- UI snapshots;
- accessibility tests;
- input tests.

Sensitive data requires explicit diagnostic permission.

## 8.63 SDK and tooling

Plugin SDK/CLI provides:

- Rust bindings;
- ABI glue generation;
- manifest schema generation;
- contribution/service/resource/migration helpers;
- local test host;
- mock capabilities;
- ZIP/TAR packaging;
- compatibility validation;
- UI IR inspector;
- contract inspector;
- native/WASM/process templates.

The low-level API remains minimal and stable.

## 8.64 Multiple runtimes

`PluginManager` is not necessarily a global singleton.

Potential sharing:

- package catalog;
- loaded native libraries;
- executors where policy permits.

Contributions always bind to a specific host application and `UiRuntime`.

Service scopes:

- process;
- application;
- runtime;
- instance.

UI handles and retained subtrees do not move between runtimes.

Cross-runtime communication uses services/messages.

## 8.65 Protected host zones

Plugins cannot replace:

- permission dialogs;
- secret entry;
- plugin management;
- security settings;
- critical confirmations;
- host identity indicators;
- protected title/chrome.

Sensitive operations use host-owned UI that identifies the requesting plugin.

Ordinary plugin UI does not require permanent visual marking.

## 8.66 Shutdown participation

Plugins receive `shutdown_requested`.

They may request bounded time and return `ShutdownConcern`.

They cannot block shutdown indefinitely.

Host owns confirmation UI, deadline and final decision.

After deadline, forced teardown proceeds.

## 8.67 Compatibility with host applications

Manifest may declare:

- supported host application IDs;
- supported application version ranges;
- required application-specific services;
- extension-point contract versions;
- optional host features.

One package may support several hosts.

Only incompatible contributions are blocked where partial activation is possible.
