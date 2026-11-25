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

### 6.16.1 Semantic Back gesture API

The interaction layer provides `BackGesture`. An application keeps a
recognizer for its navigation surface and passes dispatched events to
`update(event, scale)` when Back is available. The scale converts physical
wheel deltas. Pointer coordinates in dispatched UI events are logical.

A trusted rightward touch starting at the leading edge of the current
left-to-right implementation, or a trusted horizontal native touchpad swipe or phased pixel-scroll
sequence, can produce a `BackGestureUpdate` with source, phase and offset. The offset is
rightward displacement in logical pixels. The first recognized update is
`Started`, further movement reports `Updated`, and the sequence ends once
with `Completed` or `Cancelled`. A platform cancellation never completes
navigation. Events before recognition return no update. The recognizer's
`offset()` is present only while a recognized gesture is active.

The defaults use a 32 logical pixel touch region, an 8 logical pixel movement
threshold, a horizontal-to-vertical ratio of 1.75, and a 72 logical pixel
commit distance. Vertical movement remains available to scrolling. A system
gesture can take priority by reserving the edge or cancelling the sequence.

Applications decide whether Back is available, use the offset to drive their
own transition, navigate only on `Completed`, and settle the preview on
`Cancelled`. Window or navigation changes can explicitly cancel the sequence.

### 6.16.2 Directional swipe and touch transforms

`SwipeGesture` reports a locked direction, logical displacement, and
contact count if available. Applications may restrict the direction and
request an exact, nonzero number of contacts. Touchscreen input defaults to
one contact and follows the centroid with multiple contacts. Native touchpad
input uses the compositor's displacement and contact count when supplied.
Phased pixel scroll provides no contact count.

`TransformGesture` combines pinch and rotate. Touchscreen input uses two
contacts and their logical midpoint. Native touchpad input uses the gesture's
initial cursor anchor. Both provide the scale ratio to the initial separation
and clockwise rotation from the initial angle in radians.

Updates for swipe, transform and Back identify their source as touchscreen,
native touchpad or phased scroll. The phased-scroll source conveys that
the underlying device is unknown. Native touchpad input is delivered through
`touchpadgesture` with the platform phase and measurements.

Both recognizers report the same gesture lifecycle as Back. These values are
input measurements. Application code assigns an action to them.

### 6.16.3 UI feedback ports

A component, application or embedding host may request a semantic cue
through independently installed haptic and sound ports. Gesture recognition
does not select a cue. The owner of the interaction selects an effect and
timing, and the platform adapter applies device capabilities and user policy.
Submission does not wait for playback. A missing adapter makes that channel
silent. Gamepad rumble uses the gamepad API.

### 6.16.4 Runtime routing

A surface can opt into the runtime touch arena with `touchgesture`.
Application handlers claim the attached contact group with `prevent_default`.
A built-in scroll pan or long press that wins cancels the losing candidate
stream before carrying out its action. The raw pointer stream remains
available for observation.

A surface can opt into the phased pixel-scroll arena with `scrollgesture`.
The application contender and the vertical scroll target are anchored at
sequence start. The winner keeps the sequence through its terminal phase.
If scrolling wins, the runtime cancels the application candidate before
applying buffered displacement. Native touchpad events keep their initial
target and contact count through completion or cancellation.

The host can deliver system predictive Back to the `systemback` binding
selected at the start of the sequence. Progress is normalized and completion
or cancellation is terminal. The window-system adapter is responsible for
forwarding the system's cancellation and handling any raw input taken over
by the system. Reserved and exclusion regions are negotiated separately
through the platform port.

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
