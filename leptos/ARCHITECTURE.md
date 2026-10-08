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
                    │   • RwSignal<TransitionManager>                        │
                    │   • Provides StoryContext (Actions & Choices)          │
                    │   • Page transition fade-out/in scheduler              │
                    │   • Browser View Transitions API integration           │
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
                │  • Save / Load triggers │    ┌─────────────────────────────┐
                │  • Engine attribution   │    │     LineView (line.rs)      │
                └────────────┬────────────┘    └──────────────┬──────────────┘
                             │                                │
            opens when active│                 renders Spans  ▼
                             ▼                 ┌─────────────────────────────┐
                ┌─────────────────────────┐    │     SpanView (span.rs)      │
                │SaveLoadModal(modal.rs)  │    │  • Modifier Bitflags        │
                │  • Autosave slot        │    │  • Custom .style() Map      │
                │  • Named slots (post-   │    │  • .cls() Class Engine      │
                │    card + base64)       │    │  • CSS Keyframe Timers      │
                └─────────────────────────┘    └─────────────────────────────┘
```

The rendering pipeline converts the immediate-mode [`View`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) produced by `Game::view()` into accessible, semantic DOM nodes inside `<main id="page" aria-live="polite">`.

Context propagation is managed via [`StoryContext`](file:///home/archr/gh/OWN/ifengine/leptos/src/context.rs#L11-L24), avoiding prop drilling for action callbacks, choice dispatchers, transition manager signals, and modal controls.

---

## 2. Story Element Support (`Object` Variants)

The frontend dispatches all 8 [`Object`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L18-L42) variants defined in `ifengine_core::view::Object` via [`ObjectView`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/object.rs#L12-L247):

| `Object` Variant | DOM Tag / Component | Features & CSS Classes |
| :--- | :--- | :--- |
| [`Object::Paragraph`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L20) | `<p class="passage-paragraph">` | Standard book paragraph rhythm (1.35rem top margin, word break). Collapses margins to `0` when child elements are empty or pending entrance delay. Carries optional `RenderData`: `":speaker"` renders dialogue (`.passage-dialogue` with `<span class="dialogue-speaker">speaker</span>`), `"m-{val}"` / `"mt-{val}"` (e.g. `"m-0"`, `"mt-1.5"`) overrides inline margin to `margin-top: 0;` (or `{val}rem`), `"choice"` renders tight choice spacing (`margin-top: 0.6rem; line-height: 1.6;` on `[data-render="choice"]`), and `"popup"`/`"modal"` wraps the paragraph in a modal dialog (`.passage-popup-backdrop` / `.passage-popup-dialog`). |
| [`Object::Heading`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L28) | `<h1..h6 class="passage-heading h{level}">` | Serif headings with scaled vertical margins (`h1`=2.1rem, `h2`=1.7rem, `h3`=1.4rem, `h4`=1.2rem, `h5`=1.05rem, `h6`=0.95rem). Encapsulates [`SpanView`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/span.rs#L15-L114). |
| [`Object::Image`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L26) | `<div class="passage-image-wrapper">` | Responsive centering, max-width/max-height constraints, interactive click actions, view transition names. (See [Section 3](#3-image--media-handling)). |
| [`Object::Choice`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L24) | `<div class="passage-choices choice-container">` | Flex column list of selectable options. Renders `<button>` for navigation choices or `<div>` for inline interactive links. (See [Section 4](#4-choice-system--smart-container-delay)). |
| [`Object::Break`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L30) | `<hr class="passage-break">` | Center-aligned section divider displaying Chapbook-style triple asterisks (`*  *  *`) via `::after`. |
| [`Object::Empty`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L32) | `<div class="passage-empty">` | Configurable vertical whitespace spacer sized dynamically as `style="height: {n * 1.5}em;"`. |
| [`Object::Note`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L38) | `<aside class="passage-note">` | Dashed border callout (`1px dashed var(--color-border)`) with padding and subtle background (`rgba(255, 255, 255, 0.03)`). Renders child [`LineView`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/line.rs#L13-L71). |
| [`Object::Embed`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L41) | `<div class="passage-embed" data-page="..." data-render="...">` | Embeds sub-page views with optional `RenderData`. When `data-render` is `"popup"` or `"modal"`, wraps the embedded view in `.passage-popup-backdrop` and `.passage-popup-dialog`. An empty embed (`inner.is_empty()`) receives `.passage-custom` and serves as an extensible custom marker. |

---

## 3. Image & Media Handling

[`Object::Image`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L26) wraps the [`Image`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/image.rs#L8-L14) structure and is handled by [`ObjectView`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/object.rs#L121-L164):

1. **Sources & Variants**:
   - [`ImageVariant::Url(url)`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/image.rs#L20): Uses the string as the `src` attribute directly.
   - [`ImageVariant::Local(uri, _bytes)`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/image.rs#L21): Resolves using the static URI string.
2. **Dimension Constraints**:
   - If `width > 0`: Sets `max-width: {width}px; width: 100%;`.
   - If `height > 0`: Sets `max-height: {height}px;`.
   - Images default to responsive display (`display: block; margin: 0 auto; height: auto;`).
3. **Interactive Actions**:
   - When `img.action` is `Some(action)`, the `<img>` is wrapped within `<button type="button" class="image-action-button">`.
   - Clicking dispatches `ctx.dispatch_action.run(action.clone())` via [`StoryContext`](file:///home/archr/gh/OWN/ifengine/leptos/src/context.rs#L11-L24).
4. **Transition Integration**:
   - The outer wrapper `<div class="passage-image-wrapper">` carries `view-transition-name` and transition styles.

---

## 4. Choice System & Smart Container Delay

Interactive choices are managed by [`ChoiceView`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/choice.rs#L72-L124):

1. **Option Formatting & Interaction**:
   - Standard choices render as `<button type="button" class="choice-item choice-button">`. Clicking dispatches `ctx.dispatch_choice.run((key, choice_idx))`.
   - Choices containing embedded span actions (e.g. inline links or mutations) render as `<div class="choice-item choice-item-inline" role="button" tabindex="0">`.
   - Descendant spans inside inline choices receive [`ActiveChoiceContext`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/choice.rs#L11-L14). Clicking an action span triggers `ctx.dispatch_choice_action.run(((key, index), action))` to commit both choice selection and the span action atomically.
2. **View Transition Anchoring**:
   - Individual choice options carry `view-transition-name: item-{key}-{choice_idx}; view-transition-class: reflow;`.
3. **Smart Delay Synchronization**:
   - When all choices specify entrance delays (e.g. `choice!(s!("Option A").cls("in-500"), s!("Option B").cls("in-800"))`), the wrapper container hides (`display: none`) until the minimum delay (`500ms`) elapses, avoiding premature whitespace gaps.

---

## 5. Class System & Layout Utilities

CSS classes added through DSL builders (`.cls("...")`, `.classes([...])`) on [`Line`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/line.rs) and [`Span`](file:///home/archr/gh/OWN/ifengine/ifengine_core/src/view/line.rs) are converted to native DOM class attributes via [`span_classes`](file:///home/archr/gh/OWN/ifengine/leptos/src/render.rs#L53-L72) and [`LineView`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/line.rs#L13-L71).

### Supported Utility & State Classes

| Class Name | Target Element | Rendered Effect | Typical Story Usage |
| :--- | :--- | :--- | :--- |
| `.center` | `Line`, `Span`, `Heading` | `display: block; text-align: center;` | Title cards, chapter intros, dramatic utterances, centered symbols. |
| `.passage-link` | `Span` (link variant + action) | Clickable anchor styling with underline offset, hover/active states. | Created automatically by link builders (`link!`, `tun!`, `back!`). |
| `.passage-action` | `Span` (action without link variant) | Clickable anchor styling identical to `.passage-link`. | Interactive action spans (`alts!`, `click!`, `count!`). |
| `.passage-link-static` | `Span` (link variant without action) | Underlined link appearance without click handlers. | Static link formatting (`SpanVariant::Link`). |
| `.variant-link` | `Span` | Marks spans with link variant. | Added automatically by [`span_classes`](file:///home/archr/gh/OWN/ifengine/leptos/src/render.rs#L53-L72). |
| `.variant-muted` | `Span` | Muted color tone (`--color-text-muted`, ~65% opacity). | Meta text, side remarks, subdued thoughts. |
| `.variant-secondary` | `Span` | Secondary text color (`--color-text-dim`, `#adb5bd`). | Descriptive annotations, inventory details. |
| `.interactive-action` | `Span` | Marks spans carrying an interactive callback. | Added automatically by [`span_classes`](file:///home/archr/gh/OWN/ifengine/leptos/src/render.rs#L53-L72). |
| `.skip-animation` | `Line`, `Span` | `animation: none !important; animation-duration: 0s !important;` | Attached when an item was previously rendered to prevent replay. |
| `.delayed-hidden` | `Line`, `Span` | `display: none !important; margin: 0 !important; padding: 0 !important;` | Applied during entrance delay to prevent layout shifts. |
| `.fade-out-removed` | `Line`, `Span` | `display: none !important; margin: 0 !important; padding: 0 !important;` | Applied after an exit fade-out completes to collapse DOM space. |
| `.transition-active` | `Line`, `Span` | `will-change: opacity;` | Applied while CSS keyframe opacity transitions are active. |

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

The frontend includes a transition parser and lifecycle manager in [`transition.rs`](file:///home/archr/gh/OWN/ifengine/leptos/src/transition.rs):

### 1. Transition Class Syntax

Elements declare entrance and exit transitions directly in their class list:

| Pattern | Behavior | Defaults |
| :--- | :--- | :--- |
| `"in"` | Fade in immediately | [`DEFAULT_FADE_IN`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs#L5) (1000ms), delay = 0ms |
| `"in-{d}"` | Fade in after `{d}` ms delay | Duration = [`DEFAULT_FADE_IN`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs#L5) (1000ms), delay = `{d}` ms |
| `"in-{d}-{t}"` | Fade in after `{d}` ms delay over `{t}` ms | Duration = `{t}` ms, delay = `{d}` ms |
| `"out"` | Fade out immediately | [`DEFAULT_FADE_OUT`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs#L9) (1000ms), delay = 0ms |
| `"out-{d}"` | Fade out after `{d}` ms delay | Duration = [`DEFAULT_FADE_OUT`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs#L9) (1000ms), delay = `{d}` ms |
| `"out-{d}-{t}"` | Fade out after `{d}` ms delay over `{t}` ms | Duration = `{t}` ms, delay = `{d}` ms |

### 2. Multi-Phase Transition Lifecycle ([`TransitionPhase`](file:///home/archr/gh/OWN/ifengine/leptos/src/transition.rs#L202-L209))

To avoid layout jumps before delayed animations trigger, elements step through three phases:

```
                  ┌──────────────────────────────────────────────┐
                  │ 1. Pending (in_delay > 0)                    │
                  │ • class: delayed-hidden                      │
                  │ • style: display: none !important            │
                  │ • Parent container collapsed                 │
                  └──────────────────────┬───────────────────────┘
                                         │ timer: in_delay ms
                                         ▼
                  ┌──────────────────────────────────────────────┐
                  │ 2. Active                                    │
                  │ • class: transition-active                   │
                  │ • Keyframe fade-in (and/or fade-out) runs    │
                  │ • animation-fill-mode: both                  │
                  └──────────────────────┬───────────────────────┘
                                         │ timer: total_ms
                                         ▼
                  ┌──────────────────────────────────────────────┐
                  │ 3. Removed (if fade_out configured)          │
                  │ • class: fade-out-removed                    │
                  │ • style: display: none !important            │
                  │ • Element collapsed from layout              │
                  └──────────────────────────────────────────────┘
```

### 3. Anti-Retrigger Protection ([`TransitionManager`](file:///home/archr/gh/OWN/ifengine/leptos/src/transition.rs#L147-L198))
- [`TransitionManager`](file:///home/archr/gh/OWN/ifengine/leptos/src/transition.rs#L147-L198) tracks `(PageId, PageKey)` pairs alongside 64-bit content hashes.
- **Fresh Page Navigations** (`game.fresh() == true`): Element entrance animations are suppressed (`is_fresh == true`), deferring visual page transitions to the full-page fade-in/out wrapper.
- **Same-Page Interactions** (`game.fresh() == false`):
  1. If an element was already rendered with the same hash, its animation is suppressed (`.skip-animation`).
  2. If an element's hash changed (e.g. dynamic text altered via `alts!`), the transition re-triggers.
  3. Newly revealed items (not previously rendered on this page) animate.

---

## 8. View Transitions API Integration

[`ifengine_leptos`](file:///home/archr/gh/OWN/ifengine/leptos/src/lib.rs) leverages the browser's native [View Transitions API](https://developer.mozilla.org/en-US/docs/Web/API/View_Transitions_API) for seamless same-page DOM morphing:

### 1. Interaction Execution ([`run_with_view_transition`](file:///home/archr/gh/OWN/ifengine/leptos/src/app_impl.rs#L237-L258))
When executing same-page interactions without active modals:
- Invokes `document.startViewTransition(callback)` via `wasm-bindgen` when available.
- Falls back to direct state updates when unsupported or when modal dialogs are active.

### 2. Transition Tagging ([`generate_view_transition_style`](file:///home/archr/gh/OWN/ifengine/leptos/src/transition.rs#L322-L329))
Elements with a [`PageKey`](file:///home/archr/gh/OWN/ifengine/leptos/src/context.rs#L1) receive deterministic inline CSS transition properties:
- `view-transition-name: item-{key};`
- `view-transition-class: fade;` if content changed (`is_changed == true`). Old snapshots fade out over 0.3s, new snapshots fade in over 0.2s.
- `view-transition-class: reflow;` if content is unchanged (`is_changed == false`). Old snapshots are suppressed immediately to prevent ghosting while elements smoothly glide into new positions.

Root/viewport transitions are disabled (`html, :root { view-transition-name: none; }`) so transitions affect only story elements.

---

## 9. Persistence & Save/Load Modal Architecture

Game state persistence is handled by [`storage.rs`](file:///home/archr/gh/OWN/ifengine/leptos/src/storage.rs) and the [`SaveLoadModal`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/save_load_modal.rs#L15-L205) component:

### 1. Storage Pipeline
- **Data Structure**: [`SaveSlot<C>`](file:///home/archr/gh/OWN/ifengine/leptos/src/storage.rs#L9-L15) contains slot ID, display name, timestamp, current page ID, and the full `ifengine::Game<C>` state.
- **Serialization**: Compact binary encoding using `postcard::to_allocvec`, encoded into standard `base64` strings for browser `localStorage`.
- **Autosave**: On every page view refresh in [`StoryApp`](file:///home/archr/gh/OWN/ifengine/leptos/src/app_impl.rs#L87-L124), [`save_autosave`](file:///home/archr/gh/OWN/ifengine/leptos/src/storage.rs#L41-L59) records state under `ifengine_autosave` ([`AUTOSAVE_KEY`](file:///home/archr/gh/OWN/ifengine/leptos/src/storage.rs#L4)).
- **Manual Slots**: Stored under `ifengine_saves` ([`SAVES_KEY`](file:///home/archr/gh/OWN/ifengine/leptos/src/storage.rs#L5)). Supports creating, renaming, overwriting, and deleting slots.

### 2. Unified Modal Interface ([`SaveLoadModal`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/save_load_modal.rs#L15-L205))
Opened via [`StoryContext`](file:///home/archr/gh/OWN/ifengine/leptos/src/context.rs#L11-L24) in either [`ModalMode::Save`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/save_load_modal.rs#L9) or [`ModalMode::Load`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/save_load_modal.rs#L10):
- **Save Mode**: Displays current autosave timestamp, slot rename inputs, overwrite buttons, delete buttons, and a "+ New Save" button.
- **Load Mode**: Clickable links for the autosave slot and manual save slots, plus delete buttons.
- **Interaction**: Clicking the backdrop closes the modal; clicks inside the dialog are stopped from propagating.

---

## 10. Status Header, Footer & Theming Architecture

### 1. Status Header ([`Header`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/header.rs#L5-L26))
Renders metadata above story passages extracted via [`HeaderExtractor<C>`](file:///home/archr/gh/OWN/ifengine/leptos/src/app_impl.rs#L21):
- **Columns**: `.header-left`, `.header-center`, and `.header-right` (e.g. Day, Miles, Rations).
- **Auto-Hiding**: If no header items are produced, the component returns `None`, omitting `<header>` from the DOM entirely.

### 2. Passage Footer ([`Footer`](file:///home/archr/gh/OWN/ifengine/leptos/src/components/footer.rs#L7-L42))
Renders at the bottom of the reading column:
- **Left Actions**: "Save" and "Load" buttons invoking modal dispatchers in [`StoryContext`](file:///home/archr/gh/OWN/ifengine/leptos/src/context.rs#L11-L24).
- **Attribution**: "Made with IfEngine" link referencing the engine repository.

### 3. Design Tokens & CSS Variables
Centrally configured in [`public/style.css`](file:///home/archr/gh/OWN/ifengine/leptos/public/style.css) and [`consts.rs`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs):

| CSS Variable / Rust Constant | Default Value | Description |
| :--- | :--- | :--- |
| `--color-backdrop` / `--color-page-bg` | `#1A1B1D` | Canvas and page background color. |
| `--color-text` | `#F1F3F5` | Primary body font color. |
| `--color-text-dim` | `#ADB5BD` | Secondary text color. |
| `--color-text-muted` | `#868E96` | Muted notes, status markers, and borders. |
| `--color-link` / `--color-link-hover` | `#F1F3F5` / `#FFFFFF` | Interactive link foreground colors. |
| `--font-serif` | Iowan Old Style, Constantia, Georgia, serif | Primary passage reading typography. |
| `--font-sans` | system-ui, -apple-system, Segoe UI, sans-serif | Header, button, and metadata typography. |
| `--page-width` / [`MAX_PAGE_WIDTH_REM`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs#L24) | `44rem` | Maximum reading column width. |
| [`DEFAULT_FADE_IN`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs#L5) | `1000ms` | Default animation duration for `"in"` class. |
| [`DEFAULT_FADE_OUT`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs#L9) | `1000ms` | Default animation duration for `"out"` class. |
| [`PAGE_TRANSITION_OUT`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs#L12) | `300ms` | Full-page transition fade-out duration. |
| [`PAGE_TRANSITION_IN`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs#L15) | `200ms` | Full-page transition fade-in duration. |
| [`ELEMENT_TRANSITION_OUT`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs#L18) | `300ms` | View transition fade-out duration for modified elements. |
| [`ELEMENT_TRANSITION_IN`](file:///home/archr/gh/OWN/ifengine/leptos/src/consts.rs#L21) | `200ms` | View transition fade-in duration for modified elements. |
