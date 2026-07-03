# 6. Platform, Windows and System Services

## 6.1 Window model

Cross-platform multi-window API:

- create/destroy;
- lifecycle;
- owner/parent relations;
- modality;
- focus;
- z-order;
- inter-window state/events.

WASM may map these concepts to:

- browser windows;
- popups;
- overlays;
- virtual windows.

## 6.2 Dynamic window properties

Potential properties:

- size;
- min/max constraints;
- position;
- decorations;
- resizable;
- fullscreen;
- maximized/minimized;
- always-on-top;
- transparency;
- blur/backdrop effect;
- presentation state.

Capability result categories:

- native;
- emulated;
- conditional;
- unsupported.

Operations return typed results.

## 6.3 Activation and focus

The application may request:

1. window activation;
2. element focus.

Possible results:

- activated;
- denied;
- unsupported;
- failed.

Focus-stealing restrictions are normal platform behavior, not toolkit failure.

## 6.4 Custom chrome

Supports client-side decorations and custom title bars.

Semantic regions:

- drag;
- resize;
- close;
- minimize;
- maximize;
- system menu;
- double-click action.

Safe areas and system-frame fallback are supported.

Dynamic switching is allowed where the platform supports it.

## 6.5 Fullscreen and presentation

Modes may include:

- normal;
- borderless;
- exclusive where available;
- kiosk;
- presentation.

Optional controls:

- hide cursor;
- inhibit sleep;
- choose monitor;
- restore previous state;
- safe escape policy.

The application owns policy.

## 6.6 Displays

Display model:

- monitor list;
- logical bounds;
- physical bounds where available;
- scale/DPI;
- refresh rate;
- color/HDR capability;
- primary/current display;
- safe/reserved regions;
- hotplug/configuration events.

On Wayland, unavailable global coordinates are represented honestly.

## 6.7 Menus and actions

Unified Action model:

- stable ID;
- title;
- icon;
- shortcut;
- enabled;
- checked;
- visible;
- handler/command.

The same action can drive:

- native menu;
- kit-rendered menu;
- toolbar;
- command palette;
- tray;
- accessibility;
- shortcuts.

Fully custom context menus are required.

## 6.8 Tray/status item

Optional status item:

- icon;
- menu;
- activation callbacks;
- custom kit popup/window where platform permits;
- fallback native menu.

## 6.9 Dialog providers

Replaceable `DialogProvider` supports:

- native dialogs;
- portal/external dialogs;
- kit-rendered dialogs.

Provider chains include origin and loop detection so a system dialog implemented using Obscur Kit does not recursively call itself.

## 6.10 Document access

File pickers return `FileAccess`/`DocumentHandle`, not necessarily filesystem paths.

A document handle may provide:

- metadata;
- read bytes;
- async stream;
- write/save/create;
- persistent permission token;
- optional path hint.

The logical contract works with local files, portals, cloud documents and virtual filesystems.

## 6.11 Shell integration

`ShellProvider` may open:

- file;
- URI;
- document handle.

Results:

- opened;
- denied;
- unsupported;
- failed.

Application/installer responsibilities:

- file associations;
- MIME registration;
- protocol registration;
- manifests;
- single-instance policy.

The toolkit provides typed activation events for files, URIs and deep links.

## 6.12 Notifications and attention

Capabilities may include:

- system notifications;
- notification actions;
- badges;
- progress;
- request attention;
- activate target window/screen.

Permission and host policy are explicit.

## 6.13 Router

Basic URL-like router:

- path;
- query;
- fragment;
- route patterns;
- typed params;
- nested routes;
- history;
- back/forward;
- deep-link mapping;
- typed route state;
- route-to-screen/component mapping.

Application navigation policy remains outside the toolkit.

## 6.14 Application lifecycle hooks

Normalized hooks may include:

- startup;
- activation;
- open file;
- open URI;
- suspend;
- resume;
- shutdown;
- window lifecycle.

The kit does not own application policy or process lifetime.

## 6.15 Insets and virtual keyboard

Typed environment values:

- safe areas;
- cutouts;
- virtual keyboard geometry;
- overlay/resize/pan behavior;
- dynamic changes.

CSS environment values may expose safe-area insets.

Focused inputs may be scrolled into view.

## 6.16 Mobile gestures

Supports:

- semantic Back;
- predictive-back progress;
- edge/reserved gesture regions;
- gesture arena;
- haptics;
- exclusion regions where allowed.

System gestures have priority by default.

## 6.17 Capture and privacy

Capabilities:

- capture window;
- capture surface;
- capture node;
- capture image;
- mark privacy-sensitive/capture-protected regions;
- hide from task switcher/previews/sharing where possible.

Results:

- captured;
- denied;
- unsupported;
- failed.

## 6.18 Optional media devices

Potential optional services:

- camera;
- microphone;
- screen capture;
- device enumeration;
- hotplug;
- permissions;
- formats;
- resolution;
- FPS;
- audio formats;
- async streams;
- device loss;
- privacy indicators.

Recording, encoding and calls remain external systems.

## 6.19 Optional media playback

Potential optional playback support:

- UI sounds;
- audio/video streams;
- play/pause;
- seek;
- volume;
- output device;
- buffering;
- end/error;
- system controls;
- video synchronization.

Complex codecs and full media pipelines remain external.

## 6.20 Data transfer

Clipboard and DnD share a common `DataTransfer` subsystem:

- `DataOffer`;
- typed internal payload;
- MIME descriptors;
- multiple representations;
- lazy async providers.

Formats may include:

- plain text;
- rich text;
- URI;
- files/documents;
- image;
- custom types.

Capabilities, permissions, limits and copy/cut/paste commands are integrated.

System clipboard history is not a core feature.

## 6.21 Drag and drop

DnD supports:

- internal;
- inter-window;
- system;
- typed source/target;
- previews;
- enter/leave/over/drop;
- effects;
- cancel;
- autoscroll;
- selection;
- accessibility.

One drag may expose both internal typed payload and system MIME.

Wayland/system-crossable data should prefer system DnD.

DnD may create a provisional layout transaction for placeholders, insertion slots and reorder previews.
