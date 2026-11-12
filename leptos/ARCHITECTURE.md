# Architecture & Frontend Capabilities Specification

This document provides a comprehensive specification of [`ifengine_leptos`](file:///home/archr/gh/OWN/ifengine/leptos/src/lib.rs), the DOM/WebAssembly frontend for the `ifengine` interactive fiction engine. It focuses on the frontend presentation architecture: rendered story elements, visual styling, utility classes, inline style maps, modifier bitflags, images, interactive choices, view transitions, persistence, and browser animations.

---

## 1. Architectural Overview

[`ifengine_leptos`](file:///home/archr/gh/OWN/ifengine/leptos/src/lib.rs) provides a reactive web client built on [Leptos 0.8](https://leptos.dev) CSR (Client-Side Rendering) compiled to WebAssembly via Trunk.

```
                    ┌────────────────────────────────────────────────────────┐
                    │                 ifengine Core Engine                   │
                    │       Game<C> (Immediate-Mode Page Evaluation)         │
                    └───────────────────────────┬────────────────────────────┘
                                                │ produces View
                                                ▼
                    ┌────────────────────────────────────────────────────────┐
                    │                 App Component (app.rs)                 │
                    │       Initializes story game instance & header         │
                    └───────────────────────────┬────────────────────────────┘
                                                │ renders
                                                ▼
                    ┌────────────────────────────────────────────────────────┐
                    │               StoryApp<C> (app_impl.rs)                │
                    │   • TransitionHandle (runtime + plan + scheduler)      │
                    │   • Provides StoryContext (Actions & Choices)          │
                    │   • Page transition fade-out/in (CSS keyframes)        │
                    │   • Backend-selected same-page effects (vt/css/instant)│
                    │   • LocalStorage autosave triggers                     │
                    └────────┬──────────────────────┬────────────────────────┘
                             │                      │
             renders Header  │                      │ renders Objects
                             ▼                      ▼
                ┌─────────────────────────┐    ┌─────────────────────────────┐
                │   Header (header.rs)    │    │   ObjectView (object.rs)    │
                │  • Left / Center /      │    │  • Paragraph (dialogue/m-*) │
                │    Right status values  │    │  • Image (URL / Local)      │
                │  • Suppressed if empty  │    │  • Choice Container         │
                └─────────────────────────┘    │  • Heading (h1-h6)          │
                             │                 │  • Break / Empty / Note     │
              renders Footer │                 │  • Embed (sub-page / modal) │
                             ▼                 └──────────────┬──────────────┘
                ┌─────────────────────────┐                   │
                │   Footer (footer.rs)    │    renders Lines  ▼
                │  • Save/Load/Options    │    ┌─────────────────────────────┐
                │  • Engine attribution   │    │     LineView (line.rs)      │
                └────────────┬────────────┘    └──────────────┬──────────────┘
                             │                                │
            opens when active│                 renders Spans  ▼
                             ▼                 ┌─────────────────────────────┐
                ┌─────────────────────────┐    │     SpanView (span.rs)      │
                │ SaveLoadModal /         │    │  • Modifier Bitflags        │
                │ OptionsModal            │    │  • Custom .style() Map      │
                │  • Autosave / named     │    │  • .cls() Class Engine      │
                │    slots (postcard)     │    │  • CSS Keyframe Timers      │
                │  • Motion backend       │    │  • data-vt-* Attributes     │
                └─────────────────────────┘    └─────────────────────────────┘
```

The rendering pipeline converts the immediate-mode [`View`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) produced by `Game::view()` into accessible, semantic DOM nodes inside `<main id="page" aria-live="polite">`.

Context propagation is managed via [`StoryContext`](file:///home/archr/gh/OWN/ifengine/leptos/src/context.rs#L11-L25), avoiding prop drilling for action callbacks, choice dispatchers, transition manager signals, and modal controls.

---

## 2. Story Element Support (`Object` Variants)

The frontend dispatches all 8 [`Object`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L20-L43) variants defined in `ifengine_core::view::Object` via [`ObjectView`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/object.rs#L12):

| `Object` Variant | DOM Tag / Component | Features & CSS Classes |
| :--- | :--- | :--- |
| [`Object::Paragraph`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L22) | `<p class="passage-paragraph">` | Standard book paragraph rhythm (1.35rem top margin, word break). Collapses margins to `0` when child elements are empty or pending entrance delay. Carries optional `RenderData`: `":speaker"` renders dialogue (`.passage-dialogue` with `<span class="dialogue-speaker">speaker</span>`), `"m-{val}"` / `"mt-{val}"` (e.g. `"m-0"`, `"mt-1.5"`) overrides inline margin to `margin-top: 0;` (or `{val}rem`), `"choice"` renders tight choice spacing (`margin-top: 0.6rem; line-height: 1.6;` on `[data-render="choice"]`), and `"popup"`/`"modal"` wraps the paragraph in a modal dialog (`.passage-popup-backdrop` / `.passage-popup-dialog`). |
| [`Object::Heading`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L30) | `<h1..h6 class="passage-heading h{level}">` | Serif headings with scaled vertical margins (`h1`=2.1rem, `h2`=1.7rem, `h3`=1.4rem, `h4`=1.2rem, `h5`=1.05rem, `h6`=0.95rem). Encapsulates [`SpanView`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/span.rs#L15). |
| [`Object::Image`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L28) | `<div class="passage-image-wrapper">` | Responsive centering, max-width/max-height constraints, interactive click actions, view transition names. (See [Section 3](#3-image--media-handling)). |
| [`Object::Choice`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L26) | `<div class="passage-choices choice-container">` | Flex column list of selectable options. Renders `<button>` for navigation choices or `<div>` for inline interactive links. (See [Section 4](#4-choice-system--smart-container-delay)). |
| [`Object::Break`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L32) | `<hr class="passage-break">` | Center-aligned section divider displaying Chapbook-style triple asterisks (`*  *  *`) via `::after`. |
| [`Object::Empty`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L34) | `<div class="passage-empty">` | Configurable vertical whitespace spacer sized dynamically as `style="height: {n * 1.5}em;"`. |
| [`Object::Note`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L40) | `<aside class="passage-note">` | Dashed border callout (`1px dashed var(--color-border)`) with padding and subtle background (`rgba(255, 255, 255, 0.03)`). Renders child [`LineView`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/line.rs#L15). |
| [`Object::Embed`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L43) | `<div class="passage-embed" data-page="..." data-render="...">` | Embeds sub-page views with optional `RenderData`. When `data-render` is `"popup"` or `"modal"`, wraps the embedded view in `.passage-popup-backdrop` and `.passage-popup-dialog`. An empty embed (`inner.is_empty()`) receives `.passage-custom` and serves as an extensible custom marker. |

---

## 3. Image & Media Handling

[`Object::Image`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L28) wraps the [`Image`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/image.rs#L11) structure and is handled by [`ObjectView`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/object.rs#L133):

1. **Sources & Variants**:
   - [`ImageVariant::Url(url)`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/image.rs#L23): Uses the string as the `src` attribute directly.
   - [`ImageVariant::Local(uri, _bytes)`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/image.rs#L24): Resolves using the static URI string.
2. **Dimension Constraints**:
   - If `width > 0`: Sets `max-width: {width}px; width: 100%;`.
   - If `height > 0`: Sets `max-height: {height}px;`.
   - Images default to responsive display (`display: block; margin: 0 auto; height: auto;`).
3. **Interactive Actions**:
   - When `img.action` is `Some(action)`, the `<img>` is wrapped within `<button type="button" class="image-action-button">`.
   - Clicking dispatches `ctx.dispatch_action.run(action.clone())` via [`StoryContext`](file:///home/archr/gh/OWN/ifengine/leptos/src/context.rs#L11-L25).
4. **Transition Integration**:
   - The outer wrapper `<div class="passage-image-wrapper">` carries the plan-resolved `data-vt-*` attributes and the backend style for `view-transition-name`.

---

## 4. Choice System & Smart Container Delay

Interactive choices are managed by [`ChoiceView`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/choice.rs#L69):

1. **Option Formatting & Interaction**:
   - Standard choices render as `<button type="button" class="choice-item choice-button">`. Clicking dispatches `ctx.dispatch_choice.run((key, choice_idx))`.
   - Choices containing embedded span actions (e.g. inline links or mutations) render as `<div class="choice-item choice-item-inline" role="button" tabindex="0">`.
   - Descendant spans inside inline choices receive [`ActiveChoiceContext`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/choice.rs#L11-L14). Clicking an action span triggers `ctx.dispatch_choice_action.run(((key, index), action))` to commit both choice selection and the span action atomically.
2. **View Transition Anchoring**:
   - Choice option arms derive span keys from the choice key and arm index (`choice_item_parent_id`); the container itself is tracked for change detection but carries no `view-transition-name`.
3. **Smart Delay Synchronization**:
   - When every option carries an entrance delay (e.g. `choice!(s!("Option A").cls("in-500"), s!("Option B").cls("in-800"))`) *and* the container is newly revealed, the wrapper hides (`display: none`) until the minimum delay (`500ms`) elapses, avoiding premature whitespace gaps. Each option then keeps its own authored delay, so the pair above appears with the wrapper and staggers `300ms` later.

---

## 5. Class System & Layout Utilities

CSS classes added through DSL builders (`.cls("...")`, `.classes([...])`) on [`Line`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/line.rs) and [`Span`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/line.rs) are converted to native DOM class attributes via [`span_classes`](file:///home/archr/gh/OWN/ifengine/leptos/src/render.rs#L53-L72) and [`LineView`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/line.rs#L15).

### Supported Utility & State Classes

| Class Name | Target Element | Rendered Effect | Typical Story Usage |
| :--- | :--- | :--- | :--- |
| `.center` | `Line`, `Span`, `Heading` | `display: block; text-align: center;` | Title cards, chapter intros, dramatic utterances, centered symbols. |
| `.passage-link` | `Span` (link variant + action) | Clickable anchor styling with underline offset, hover/active states. | Created automatically by link builders (`link!`, `tun!`, `back!`). |
| `.passage-action` | `Span` (action without link variant) | Clickable anchor styling identical to `.passage-link`. | Interactive action spans (`alts!`, `click!`). |
| `.passage-link-static` | `Span` (link variant without action) | Underlined link appearance without click handlers. | Static link formatting (`SpanVariant::Link`). |
| `.variant-link` | `Span` | Marks spans with link variant. | Added automatically by [`span_classes`](file:///home/archr/gh/OWN/ifengine/leptos/src/render.rs#L53-L72). |
| `.variant-muted` | `Span` | Muted color tone (`--color-text-muted`, ~65% opacity). | Meta text, side remarks, subdued thoughts. |
| `.variant-secondary` | `Span` | Secondary text color (`--color-text-dim`, `#adb5bd`). | Descriptive annotations, inventory details. |
| `.interactive-action` | `Span` | Marks spans carrying an interactive callback. | Added automatically by [`span_classes`](file:///home/archr/gh/OWN/ifengine/leptos/src/render.rs#L53-L72). |
| `.delayed-hidden` | `Line`, `Span` | `display: none !important; margin: 0 !important; padding: 0 !important;` | Applied during entrance delay to prevent layout shifts. |
| `.fade-out-removed` | `Line`, `Span` | Fade-out keyframe over `--vt-fade-out` (or `--default-fade-out`). | Applied while an authored `out-*` exit fades. |
| `.transition-removed` | `Line`, `Span` | `display: none !important;` | Applied after the exit fade completes. |
| `.transition-active` | `Line`, `Span` | `will-change: opacity;` | Applied while CSS keyframe opacity transitions are active. |

`.variant-link`, `.interactive-action` and `.passage-custom` are marker classes: the frontend emits them for story-authoring hooks, but the stylesheet does not target them, so they carry no styling of their own.

---

## 6. Style Tags & Inline CSS Mapping

[`ifengine_leptos`](file:///home/archr/gh/OWN/ifengine/leptos/src/lib.rs) converts span metadata to inline CSS styles via [`span_to_css_style`](file:///home/archr/gh/OWN/ifengine/leptos/src/render.rs#L6-L50).

### 1. The `.style(key, value)` Map
Story authors can set arbitrary CSS properties directly on any `Span` using `.style("property", "value")`:

```rust
s!("SALTWRACK")
    .cls("center")
    .style("display", "block")
    .style("margin-top", "clamp(6rem, 15vh, 16rem)")
    .style("letter-spacing", "0.08em")
```

[`span_to_css_style`](file:///home/archr/gh/OWN/ifengine/leptos/src/render.rs#L6-L50) maps key-value pairs into the element's inline `style="..."` attribute:
- `"color"` &rarr; `color: {v};`
- `"background"` &rarr; `background-color: {v};`
- `"font-size"` &rarr; `font-size: {v};`
- **Arbitrary CSS properties** &rarr; passed through directly as `{k}: {v};` (e.g. `margin-top`, `margin-bottom`, `letter-spacing`, `display`, `opacity`, `transform`).

### 2. Semantic Modifiers ([`Modifier`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/line.rs) Bitflags)
When modifier flags are set on a `Span`, the frontend generates corresponding CSS declarations:

| Flag | Generated Inline CSS |
| :--- | :--- |
| `Modifier::BOLD` | `font-weight: bold;` |
| `Modifier::ITALIC` | `font-style: italic;` |
| `Modifier::DIM` | `opacity: 0.65;` |
| `Modifier::UNDERLINE` | `text-decoration: underline; text-underline-offset: 3px;` |
| `Modifier::STRIKETHROUGH` | `text-decoration: line-through;` |
| `Modifier::HIDDEN` | `display: none;` |
| `Modifier::SUPER_SCRIPT` | `vertical-align: super; font-size: 0.8em;` |
| `Modifier::SUBSCRIPT` | `vertical-align: sub; font-size: 0.8em;` |

### 3. Variants ([`SpanVariant`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/line.rs))
- `SpanVariant::Link`: Injects underline styling (`text-decoration: underline; text-underline-offset: 3px;`) and `.variant-link`.
- `SpanVariant::Muted`: Injects `opacity: 0.65;` and `.variant-muted`.
- `SpanVariant::Secondary`: Injects `color: var(--color-secondary, #adb5bd);` and `.variant-secondary`.

---

## 7. Animation & Transition Engine

Transitions are planned in Rust and effected by a selectable backend under [`transition/`](file:///home/archr/gh/OWN/ifengine/leptos/src/transition/mod.rs):

- `plan.rs` walks each rendered view once, assigning identity, content hash, `fade`/`reflow` classification, authored timing and geometry kind to every named element.
- `runtime.rs` owns change tracking and the `TransitionHandle` that components reach through `StoryContext` (`item_attrs`, `commit`, `schedule`, `reset`).
- `scheduler.rs` is the epoch/milestone state machine for delayed batches.
- `backend.rs` selects `vt.js` (View Transitions), `css.js` (clone snapshot) or an instant update.

### 1. Transition Class Syntax

Elements declare entrance and exit transitions directly in their class list. The delay decides which clock animates the item: a delay of `0` keeps it on the **primary** (same-commit) transition, a positive delay moves it to the **wait clock** (a scheduler milestone that runs as its own batch after the primary settles — see §8).

| Pattern | Behavior | Clock |
| :--- | :--- | :--- |
| `"in"`, `"in-0"`, `"in-0-{t}"` | visible in the commit that introduces it; fades in with that transition | primary |
| `"in-{d}"` | held hidden for `{d}` ms, then fades in | wait clock |
| `"in-{d}-{t}"` | held hidden for `{d}` ms, then fades in over `{t}` ms | wait clock |
| `"out"`, `"out-0"`, `"out-0-{t}"` | still rendered by the commit, starts fading out with it | primary |
| `"out-{d}"` | stays visible for `{d}` ms, then fades out | wait clock |
| `"out-{d}-{t}"` | stays visible for `{d}` ms, then fades out over `{t}` ms | wait clock |

Fade durations come from the token's `-{t}` part when authored, otherwise from `--default-fade-in` / `--default-fade-out`. Inside a secondary batch a fade instead runs at the batch's own length (`--secondary-reflow-duration`), so fades and movement end together (§8). A delay-0 exit with an authored duration means "this item is gone `{t}` ms from now": `{t}` sizes the live fade (`.fade-out-removed` reads `--vt-fade-out`) and the timeout that hides the element, so `out-0-800` fades out over 800 ms while staying in charge of its own removal.

### 2. Multi-Phase Transition Lifecycle ([`TransitionPhase`](file:///home/archr/gh/OWN/ifengine/leptos/src/transition/mod.rs))

To avoid layout jumps before delayed animations trigger, elements step through four phases:

```
Pending --(in_delay)--> Active --(out_delay)--> Removing --(fade_out)--> Removed
delayed-hidden          transition-active       fade-out-removed           transition-removed
```

- `Pending`: `display: none`, with parent containers collapsed via `:has(.delayed-hidden)`.
- `Active`: fade-in keyframe with `animation-fill-mode: both` (or steady opacity when animation is suppressed).
- `Removing`: `fade-out-removed` fades the element in place over `fade_out` / `--default-fade-out`.
- `Removed`: `display: none`; the element no longer takes layout.

### 3. Anti-Retrigger Protection ([`TransitionManager`](file:///home/archr/gh/OWN/ifengine/leptos/src/transition/mod.rs))

- `TransitionManager` tracks `(PageId, PageKey)` pairs alongside 64-bit content hashes; the plan queries it once per item per rendered view.
- Same hash on the same page: `reflow` (geometry may slide, no flash).
- Changed hash (e.g. dynamic text altered via `alts!`): `fade`; the transition re-triggers.
- Newly seen non-zero-hash items: `fade`; zero-hash tracked containers (choices) stay `reflow`.
- Page navigation resets the scheduler epoch but keeps the seen map, so revisits do not re-fade unchanged items.

---

## 8. Backends & Same-Page Effects

`backend.rs` detects the engine once at startup and annotates `<html data-vt-backend="vt|css|instant">`:

| Engine | Backend | Rationale |
| :--- | :--- | :--- |
| Blink (Chrome, Chromium, Edge) with full capability support | `ViewTransition` | Native snapshot morphing reproduces the authored timeline exactly. |
| Gecko (Firefox, Zen) | `Css` | View Transitions capture mishandles inline elements broken across line boxes (Mozilla bug 2080048). |
| Everything else (including WebKit) and any Blink without required capabilities | `Instant` | No same-page animation; updates apply directly. |

Selection precedence is the `?vt=vt|css|instant` query parameter (QA and development), then the reader's stored preference, then the engine default above. `prefers-reduced-motion` deliberately does not participate: the reader decides, via the options modal.

[`OptionsModal`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/options_modal.rs) writes the choice through `TransitionHandle::set_backend`, which resets the outgoing backend, re-announces `data-vt-backend`, and persists the preference; item renders subscribe to the backend so inline attributes follow the switch immediately. Motion is the only option today; the modal is opened from the footer's Options button.

Components never emit `view-transition-*` CSS. During render, `ctx.transitions.item_attrs(...)` resolves a plan entry and every named element carries `data-vt-name`, `data-vt-hash`, `data-vt-effect` and `data-vt-kind`:

- **`vt.js`**: snapshots pre-update geometry and hashes from `[data-vt-name]`, normalizes each old element to the class its counterpart will use, runs the update inside `document.startViewTransition`, then classifies each element (missing counterpart, hash change, size delta or horizontal shift → `fade`; otherwise `reflow`). Classification compares the live DOM against the snapshot rather than trusting the plan's `changed` flag, which reports first sight and would also cover elements that only need to move. Explicit `in-{d}-{t}` / `out-{d}-{t}` durations become `::view-transition-{new,old}` rules generated from the plan.
- **`css.js`**: two-phase layout-projection and clone backend. Decouples measurement from mutation:
  1. *Phase 1 (Measure & Plan / Reads only)*: Pre-update snapshot (`snapshotNamed`) records baseline rects, tags, and semantic classes. Post-DOM-update, `planActions` compares current layout geometry and classifies elements into `enters` (brand-new/changed), `slides` (block containers whose position moved), `crossfades` (stationary elements whose tag or semantic styling morphed, e.g. active link $\rightarrow$ plain text), and `leaves` (departing counterparts). No styles or transforms are mutated during this phase, eliminating layout thrashing and measurement contamination.
  2. *Phase 2 (Apply & Animate / Writes only)*: Batch-applies animations. Moved block containers (`p`, `blockquote`, etc.) FLIP-slide via CSS `transform: translateY(dy)` (`.vt-slide`). Stationary morphed spans cross-fade in place (`.vt-crossfade-in` on live, `.vt-crossfade-out` on clone) with `mix-blend-mode: plus-lighter`. Brand-new tokens fade in (`.vt-enter`), respecting authored `--vt-in-delay` or `--primary-block-in-delay`. Departing clone copies fade out (`.vt-leave`). Spacers never animate.
- **`Instant`**: callbacks run immediately; no effects.

Root/viewport transitions are disabled (`html, :root { view-transition-name: none; }`), so transitions affect only story elements. Modal opens bypass transitions entirely.

### Secondary Batches (Delayed Entrances & Exits)

Delayed content runs on two independent clocks.

**Wait clock (Rust).** A `in-{d}` / `out-{d}` delay with `d > 0` is a scheduler milestone, never a CSS delay. The origin is the moment the *primary* transition finished, so `in-500` means "500 ms after the page settled", not "500 ms after the click". Milestones are absolute rather than chained, and batches are serialised (`Scheduler::active_transition`), so a deadline that has already passed runs as soon as the previous batch finishes and a slow batch delays the next one only by its real duration. A delay of `0` never reaches this clock: the item is left to the primary transition of the commit it takes part in (`in`/`in-0` is visible in that commit, `out`/`out-0` starts fading there), which is what makes an immediate exit belong to the same-commit snapshot pair instead of a batch that arrives after it.

**Animation clock (CSS/JS).** Every animation in a batch starts at delay `0s`: the wait was paid on the first clock, and re-adding it would double it. A whole batch therefore plays at once — there is no intra-batch stagger.

| Participant | Rendering | Animation | Duration |
| :--- | :--- | :--- | :--- |
| Appeared (`fade`, no old snapshot) | new snapshot | fade-in | `--secondary-reflow-duration`, or the token's `-{t}` |
| Disappeared (`fade`, old snapshot only) | old snapshot | fade-out | `--secondary-reflow-duration`, or the token's `-{t}` |
| Rewritten in place (hash or box changed) | both snapshots | in-place cross-fade | `--secondary-reflow-duration` |
| Moved (`reflow`) | old hidden, new opaque | group slide | `--secondary-reflow-duration` |

Every rule above falls back through `var(--secondary-duration, var(--secondary-reflow-duration, 1.5s))`, and `--secondary-duration` is only ever written by the opt-in stretch described below — so with the const off (the default) the whole batch runs on the reflow duration.

`group(.fade)` is `animation: none`, so fade items never slide, and movers never cross-fade, which keeps them continuously visible while they travel. Fades and the slide share one length, so a batch settles as one: `--default-fade-*` does not time these snapshots, it only reaches authored tokens through the plan (see below).

`--secondary-duration` is an opt-in stretch, gated by [`STRETCH_SECONDARY_TO_AUTHORED_FADE`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs#L14). Only when that const is on does `vt.js` write it on `<html>` for a batch, as `max(--secondary-reflow-duration, longest authored fade)` — the slide and the fades then stretch far enough to cover a fade the author asked for that would otherwise outlive them. Authored fades are the only inputs, because only they can be arbitrarily long: a delay-only token such as `in-500` counts, since the fade it will actually run resolves to `--default-fade-in`, while the batch's own reconciliation cross-fades are excluded — they are already timed by the value being computed, so folding them in would let `--default-fade-*` stretch a slide that was deliberately shortened. Departures are excluded structurally: they exist only in the old snapshot and are never visited.

The duration part of an authored `in-{d}-{t}` / `out-{d}-{t}` is injected into `<style id="vt-story-durations">` as `::view-transition-{new,old}(.<token>) { animation-duration: … }`, repeated once with an `html.secondary-transition` prefix. The duplication follows from the delay rule above rather than being a scope split: delay-0 tokens are animated by primary transitions while positive-delay tokens are animated by secondary batches, so a duration must be able to win in both. It is also a specificity matter — the token is part of the element's `view-transition-class` in either case, and the secondary block matches `.fade` at (0,2,2) `!important`, which outranks a bare pseudo-element rule at (0,1,1). The generated style element is appended after the stylesheet, so the prefixed copy wins the order tie. Only durations are generated — delays stay `0s`, because a positive wait belongs to the scheduler and a zero wait means the primary transition. Delay-only tokens such as `in-500` emit no rule and take their duration from the batch, which resolves them to `--default-fade-in` by way of the slide length; no story authors an explicit duration today, so the rule set is empty in practice and the `<style>` element stays empty.

---

## 9. Persistence & Save/Load Modal Architecture

Game state persistence is handled by [`storage.rs`](file:///home/archr/gh/OWN/ifengine/leptos/src/storage.rs) and the [`SaveLoadModal`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/save_load_modal.rs#L15-L205) component:

### 1. Storage Pipeline
- **Data Structure**: [`SaveSlot<C>`](file:///home/archr/gh/OWN/ifengine/leptos/src/storage.rs#L11) contains slot ID, display name, timestamp, current page ID, and the full `ifengine::Game<C>` state.
- **Serialization**: Compact binary encoding using `postcard::to_allocvec`, encoded into standard `base64` strings for browser `localStorage`.
- **Autosave**: On every page view refresh in [`StoryApp`](file:///home/archr/gh/OWN/ifengine/leptos/src/app_impl.rs#L169), [`save_autosave`](file:///home/archr/gh/OWN/ifengine/leptos/src/storage.rs#L43) records state under `ifengine_autosave` ([`AUTOSAVE_KEY`](file:///home/archr/gh/OWN/ifengine/leptos/src/storage.rs#L6)).
- **Manual Slots**: Stored under `ifengine_saves` ([`SAVES_KEY`](file:///home/archr/gh/OWN/ifengine/leptos/src/storage.rs#L7)). Supports creating, renaming, overwriting, and deleting slots.
- **Preferences**: [`Preferences`](file:///home/archr/gh/OWN/ifengine/leptos/src/storage.rs) holds reader settings (currently the motion backend) as JSON under `ifengine_preferences` ([`PREFERENCES_KEY`](file:///home/archr/gh/OWN/ifengine/leptos/src/storage.rs)).

### 2. Unified Modal Interface ([`SaveLoadModal`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/save_load_modal.rs#L15-L205))
Opened via [`StoryContext`](file:///home/archr/gh/OWN/ifengine/leptos/src/context.rs#L11-L25) in either [`ModalMode::Save`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/save_load_modal.rs#L9) or [`ModalMode::Load`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/save_load_modal.rs#L10):
- **Save Mode**: Displays current autosave timestamp, slot rename inputs, overwrite buttons, delete buttons, and a "+ New Save" button.
- **Load Mode**: Clickable links for the autosave slot and manual save slots, plus delete buttons.
- **Interaction**: Clicking the backdrop closes the modal; clicks inside the dialog are stopped from propagating.

---

## 10. Status Header, Footer & Theming Architecture

### 1. Status Header ([`Header`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/header.rs#L16))
Renders metadata above story passages extracted via [`HeaderExtractor<C>`](file:///home/archr/gh/OWN/ifengine/leptos/src/app_impl.rs#L26):
- **Columns**: `.header-left`, `.header-center`, and `.header-right` (e.g. Day, Miles, Rations).
- **Auto-Hiding**: If no header items are produced, the component returns `None`, omitting `<header>` from the DOM entirely.

### 2. Passage Footer ([`Footer`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/footer.rs#L7))
Renders at the bottom of the reading column:
- **Left Actions**: "Save", "Load", and "Options" buttons invoking modal dispatchers in [`StoryContext`](file:///home/archr/gh/OWN/ifengine/leptos/src/context.rs#L11-L25).
- **Attribution**: "Made with IfEngine" link referencing the engine repository.

### 3. Design Tokens & CSS Variables
Centrally configured in [`public/style.css`](file:///home/archr/gh/OWN/ifengine/leptos/public/style.css):

| CSS Variable | Default Value | Description |
| :--- | :--- | :--- |
| `--color-backdrop` / `--color-page-bg` | `#1A1B1D` | Canvas and page background color. |
| `--color-text` | `#F1F3F5` | Primary body font color. |
| `--color-text-dim` | `#ADB5BD` | Secondary text color. |
| `--color-text-muted` | `#868E96` | Muted notes, status markers, and borders. |
| `--color-link` / `--color-link-hover` | `#F1F3F5` / `#FFFFFF` | Interactive link foreground colors. |
| `--font-serif` | Iowan Old Style, Constantia, Georgia, serif | Primary passage reading typography. |
| `--font-sans` | system-ui, -apple-system, Segoe UI, sans-serif | Header, button, and metadata typography. |
| `--page-width` | `44rem` | Maximum reading column width. |
| `--page-transition-out-duration` | `0.6s` | Page navigation: fade-out before the view swaps (read back by the Rust timer). |
| `--page-transition-in-duration` | `0.4s` | Page navigation: fade-in of the new view. |
| `--primary-fade-out-duration` | `0.3s` | Primary transition: fade-out duration of departed content. |
| `--primary-reflow-offset` | `0.2s` | Primary transition: delay before reflow begins (both movers sliding and stationary links cross-fading; = fade-out duration / 2). |
| `--primary-reflow-duration` | `0.6s` | Primary transition: duration of reflow slide and stationary cross-fade morph. |
| `--primary-in-delay` | `0.4s` | Primary transition: span-level fade-in delay after reflow starts. |
| `--primary-block-in-delay` | `0.9s` | Primary transition: block-level fade-in delay for newly introduced blocks without authored timing overrides. |
| `--primary-fade-in-offset` | `0.4s` | Primary transition: delay before new content fades in, aligned to the slide's end. |
| `--primary-fade-in-duration` | `0.4s` | Primary transition: fade-in duration of new content. |
| `--secondary-reflow-duration` | `0.6s` | Secondary batch: slide *and* fades (fallback `1.5s`). |
| `--secondary-duration` | *(unset)* | Optional per-batch stretch to cover an authored fade, written by `vt.js` only when `STRETCH_SECONDARY_TO_AUTHORED_FADE` is on. |
| `--default-fade-in` | `0.6s` | Fade length for `"in"` tokens without a `-{t}` (fallback `1.5s`); also how an authored entrance enters the batch length. |
| `--default-fade-out` | `0.6s` | Same for `"out"` tokens. |

These live in `:root`; the parenthesised fallbacks are what the code uses when a variable is unset. Both the Native View Transition backend (`vt.js`) and the CSS Clone backend (`css.js`) share and respect these exact same design system knobs. The primary offsets are derived rather than independent: `reflow-offset = fade-out / 2` and `fade-in-offset = reflow - (fade-out - reflow-offset)`.
