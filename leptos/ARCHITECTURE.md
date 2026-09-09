# Architecture & Frontend Capabilities Specification

This document provides a comprehensive specification of [`ifengine_leptos`](file:///Users/absinthe/gh/OWN/ifengine/leptos), the DOM/WebAssembly frontend for the `ifengine` interactive fiction engine. It primarily focuses on **what the frontend supports**: rendered story elements, visual styling, utility classes (such as `.center`), inline style maps, modifier bitflags, images, interactive choices, transitions, and browser animations.

---

## 1. Architectural Overview

[`ifengine_leptos`](file:///Users/absinthe/gh/OWN/ifengine/leptos) provides a reactive web client built on [Leptos 0.8](https://leptos.dev) CSR (Client-Side Rendering) compiled to WebAssembly via Trunk.

```
                    ┌───────────────────────────────────────────────┐
                    │            ifengine Core Engine               │
                    │   Game<C> (Immediate-Mode Page Evaluation)    │
                    └───────────────────────┬───────────────────────┘
                                            │ produces View
                                            ▼
                    ┌───────────────────────────────────────────────┐
                    │                StoryApp (app.rs)              │
                    │   • RwSignal<TransitionManager>               │
                    │   • Action / Choice Dispatch Callbacks        │
                    │   • Browser View Transitions Integration      │
                    └───────┬───────────────────────────────┬───────┘
                            │                               │
             renders Header │                               │ renders Objects
                            ▼                               ▼
               ┌─────────────────────────┐     ┌─────────────────────────┐
               │    Header (header.rs)   │     │  ObjectView (object.rs) │
               │   • Left / Center /     │     │  • Paragraph / Text     │
               │     Right status values │     │  • Image (URL / Local)  │
               └─────────────────────────┘     │  • Choice Container     │
                                               │  • Heading (h1-h6)      │
                                               │  • Break / Empty / Note │
                                               │  • Quote / Custom/Embed │
                                               └────────────┬────────────┘
                                                            │
                                             renders Lines  ▼
                                               ┌─────────────────────────┐
                                               │    LineView (line.rs)   │
                                               └────────────┬────────────┘
                                                            │
                                             renders Spans  ▼
                                               ┌─────────────────────────┐
                                               │    SpanView (span.rs)   │
                                               │  • Modifier Bitflags    │
                                               │  • Custom .style() Map  │
                                               │  • .cls() Class Engine  │
                                               │  • CSS Keyframe Timers  │
                                               └─────────────────────────┘
```

The rendering pipeline converts the immediate-mode [`View`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) produced by `Game::view()` into accessible, semantic DOM nodes.

---

## 2. Story Element Support (`Object` Variants)

The frontend dispatches all 11 [`Object`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) variants defined in `ifengine_core::view::Object` via [`ObjectView`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/components/object.rs):

| `Object` Variant | DOM Tag / Component | Features & CSS Classes |
| :--- | :--- | :--- |
| [`Object::Paragraph`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) | `<p class="passage-paragraph">` | Standard book paragraph rhythm (1.35rem bottom margin, word break). Collapses margins to `0` when child elements are empty or pending entrance delay. |
| [`Object::Text`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) | `<div class="passage-text" data-render="...">` | Tighter vertical rhythm (`1rem`). Emits raw `data-render` attribute on DOM node. When `data-render` is `"popup"` (or `"modal"`), wraps text in `.passage-popup-backdrop` and `.passage-popup-dialog` without built-in close buttons. |
| [`Object::Heading`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) | `<h1..h6 class="passage-heading h{level}">` | Serif headings (`h1`=2.1rem, `h2`=1.7rem, `h3`=1.4rem with letter spacing, `h4`=1.2rem, `h5`=1.05rem, `h6`=0.95rem uppercase). Supports child span modifiers and styles. |
| [`Object::Image`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) | `<div class="passage-image-wrapper">` | Responsive centering, dimensions, interactive click actions, view transition names. (See Section 3). |
| [`Object::Choice`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) | `<div class="passage-choices choice-container">` | Flex column list of options. Renders `<button>` for simple choices or `<div>` for inline interactive links. (See Section 4). |
| [`Object::Break`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) | `<hr class="passage-break">` | Center-aligned section divider displaying Chapbook-style triple asterisks (`*  *  *`) via `::after`. |
| [`Object::Empty`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) | `<div class="passage-empty">` | Configurable vertical whitespace spacer sized dynamically as `style="height: {n * 1.5}em;"`. |
| [`Object::Quote`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) | `<blockquote class="passage-quote" data-render="...">` | Callout block with left accent border (`3px solid var(--color-border)`), subtle translucent background, italic styling, and muted text. |
| [`Object::Note`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) | `<aside class="passage-note">` | Dashed border callout (`1px dashed var(--color-border)`) with padding and secondary text color. |
| [`Object::Embed`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs) | `<div class="passage-embed" data-page="..." data-render="...">` | Embeds sub-page views with optional `RenderData`. When `data-render` is `"popup"` (or `"modal"`), wraps the embedded view in `.passage-popup-backdrop` and `.passage-popup-dialog` without built-in close buttons. An empty embed (`.inner.is_empty()`) receives `.passage-custom` and serves as an extensible custom marker. |

---

## 4. Class System & Layout Utilities

CSS classes added through DSL builders (`.cls("...")`, `.classes([...])`) on [`Line`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/line.rs) and [`Span`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/line.rs) are converted to native DOM class attributes via [`span_classes`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/render.rs) and [`LineView`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/components/line.rs).

### Supported Utility Classes

| Class Name | Target Element | Rendered Effect | Typical Story Usage |
| :--- | :--- | :--- | :--- |
| `.center` | `Line`, `Span`, `Heading` | `display: block; text-align: center;` | Title cards, chapter intros, dramatic utterances, centered symbols. |
| `.text-center` | `Line`, `Span` | `display: block; text-align: center;` | Alternative centering class for layout blocks. |
| `.passage-link` | `Span` (with action) | Native clickable anchor appearance with underline offset, hover/active states. | Created automatically by `link!`, `tun!`, `back!`, `click!`. |
| `.passage-link-static` | `Span` (no action) | Underlined link appearance without click handlers. | Static link formatting (`SpanVariant::Link`). |
| `.variant-muted` | `Span` | Muted color tone (`--color-text-muted`, ~65% opacity). | Meta text, side remarks, subdued thoughts. |
| `.variant-secondary` | `Span` | Secondary text color (`--color-text-dim`, `#adb5bd`). | Descriptive annotations, inventory details. |
| `.interactive-action` | `Span` | Marks spans carrying an interactive callback. | Added automatically by [`span_classes`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/render.rs). |
| `.skip-animation` | `Line`, `Span` | Enforces `animation: none !important; animation-duration: 0s !important;`. | Attached when an item was already rendered in a prior iteration to prevent animation replay. |
| `.delayed-hidden` | `Line`, `Span` | `display: none !important; margin: 0 !important; padding: 0 !important;`. | Applied during entrance delay to prevent layout jumping before fade-in. |
| `.fade-out-removed` | `Line`, `Span` | `display: none !important; margin: 0 !important; padding: 0 !important;`. | Applied after an exit fade-out completes to cleanly collapse DOM space. |
| `.transition-active` | `Line`, `Span` | `will-change: opacity;`. | Applied while CSS keyframe opacity transitions are active. |

---

## 5. Style Tags & Inline CSS Mapping

[`ifengine_leptos`](file:///Users/absinthe/gh/OWN/ifengine/leptos) provides rich CSS customization through [`span_to_css_style`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/render.rs).

### 1. The `.style(key, value)` Map
Story authors can set arbitrary CSS properties directly on any `Span` using `.style("property", "value")`:

```rust
s!("SALTWRACK")
    .cls("center")
    .style("display", "block")
    .style("margin-top", "clamp(6rem, 15vh, 16rem)")
    .style("letter-spacing", "0.08em")
```

[`span_to_css_style`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/render.rs) maps key-value pairs into the element's inline `style="..."` attribute:
- `"color"` &rarr; `color: {v};`
- `"background"` &rarr; `background-color: {v};`
- `"font-size"` &rarr; `font-size: {v};`
- **Arbitrary CSS properties** &rarr; passed through directly as `{k}: {v};` (e.g. `margin-top`, `margin-bottom`, `letter-spacing`, `display`, `opacity`, `transform`).

### 2. Semantic Modifiers ([`Modifier`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/line.rs) Bitflags)
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

### 3. Variants ([`SpanVariant`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/line.rs))
- `SpanVariant::Link`: Injects underline styling and classes.
- `SpanVariant::Muted`: Injects `opacity: 0.65;` and `.variant-muted`.
- `SpanVariant::Secondary`: Injects `color: var(--color-secondary, #adb5bd);` and `.variant-secondary`.

---

## 6. Animation & Transition Engine

The frontend includes a transition parser and lifecycle manager in [`transition.rs`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/transition.rs):

### 1. Transition Class Syntax

Elements can declare entrance and exit transitions directly in their class list:

| Pattern | Behavior | Defaults |
| :--- | :--- | :--- |
| `"in"` | Fade in immediately | `DEFAULT_FADE_IN_DURATION_MS = 1000ms`, delay = 0ms |
| `"in-{d}"` | Fade in after `{d}` ms delay | Duration = `DEFAULT_FADE_IN_DURATION_MS (1000ms)`, delay = `{d}` ms |
| `"in-{d}-{t}"` | Fade in after `{d}` ms delay over `{t}` ms | Duration = `{t}` ms, delay = `{d}` ms |
| `"out"` | Fade out immediately | `DEFAULT_FADE_OUT_DURATION_MS = 1000ms`, delay = 0ms |
| `"out-{d}"` | Fade out after `{d}` ms delay | Duration = `DEFAULT_FADE_OUT_DURATION_MS (1000ms)`, delay = `{d}` ms |
| `"out-{d}-{t}"` | Fade out after `{d}` ms delay over `{t}` ms | Duration = `{t}` ms, delay = `{d}` ms |

### 2. Multi-Phase Transition Lifecycle ([`TransitionPhase`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/transition.rs))

To avoid layout jumps before delayed animations trigger, elements step through three distinct phases:

```
                  ┌──────────────────────────────────────────────┐
                  │ 1. Pending (in_delay > 0)                    │
                  │ • class: delayed-hidden                      │
                  │ • style: display: none !important            │
                  │ • Parent paragraph collapsed                 │
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
                  │ • Element fully removed from layout          │
                  └──────────────────────────────────────────────┘
```

### 3. Anti-Retrigger Protection ([`TransitionManager`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/transition.rs))
- **Problem**: When a reader clicks an inline link or choice, the page function re-executes (`game.fresh() == false`). In naive implementations, all entrance animations replay.
- **Solution**: [`TransitionManager`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/transition.rs) records `(PageId, PageKey)` alongside content hashes.
- **Rules**:
  1. If an element has already been seen with the same hash, its animation is suppressed (`.skip-animation`, `opacity: 1; animation: none !important;`).
  2. If an element's hash changed (e.g. dynamic text altered via `alts!`), the transition re-triggers.
  3. On fresh navigation to a new page (`game.fresh() == true`), the seen cache for that page is cleared so opening entrance animations play.

---

## 7. Choice System & Smart Container Delay

Interactive choices are managed by [`ChoiceView`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/components/choice.rs):

1. **Option Formatting**:
   - Standard navigation choices render as `<button type="button" class="choice-item choice-button">`.
   - Choices containing embedded span actions render as `<div class="choice-item choice-item-inline">` so embedded interactive spans remain individually clickable.
2. **Smart Delay Synchronization**:
   - If multiple choices specify entrance delays (e.g. `choice!(s!("Choice A").cls("in-500"), s!("Choice B").cls("in-800"))`), the wrapper container is hidden (`display: none`) until the minimum delay (`500ms`) elapses. This eliminates empty whitespace gaps before choices become visible.

---

## 8. Status Header & Theming Architecture

### 1. Responsive 3-Column Header
[`Header`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/components/header.rs) renders a metadata bar above story passages:
- **Left Column** (`.header-left`): Typically displays temporal markers (e.g. `Day: 3`).
- **Center Column** (`.header-center`): Distance or journey metrics (e.g. `Miles travelled: 142`).
- **Right Column** (`.header-right`): Inventory and survival tallies (e.g. `Rations: 8`).
- **Auto-Hiding**: If no header items exist, the header receives `.header-empty` and is completely hidden (`display: none !important;`).

### 2. Design Tokens & CSS Variables
All colors, dimensions, and transition constants are centrally configurable in [`style.css`](file:///Users/absinthe/gh/OWN/ifengine/leptos/style.css) and [`consts.rs`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/consts.rs):

| CSS Variable / Rust Constant | Default Value | Description |
| :--- | :--- | :--- |
| `--color-backdrop` / `--color-page-bg` | `#1A1B1D` | Canvas and page background color. |
| `--color-text` | `#F1F3F5` | Primary body font color. |
| `--color-text-dim` | `#ADB5BD` | Secondary text color. |
| `--color-text-muted` | `#868E96` | Muted notes and dividers. |
| `--color-link` / `--color-link-hover` | `#F1F3F5` / `#FFFFFF` | Interactive link foreground colors. |
| `--font-serif` | Iowan Old Style, Georgia, serif | Primary passage reading font. |
| `--font-sans` | system-ui, Segoe UI, sans-serif | Header and UI typography. |
| `--page-width` / [`MAX_PAGE_WIDTH_REM`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/consts.rs) | `44rem` | Maximum story reading column width. |
| [`DEFAULT_FADE_IN_DURATION_MS`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/consts.rs) | `1000ms` | Default animation duration for `"in"` class. |
| [`DEFAULT_FADE_OUT_DURATION_MS`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/consts.rs) | `1000ms` | Default animation duration for `"out"` class. |
| [`DEFAULT_PAGE_TRANSITION_OUT_MS`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/consts.rs) | `300ms` | Page fade-out transition duration. |
| [`DEFAULT_PAGE_TRANSITION_IN_MS`](file:///Users/absinthe/gh/OWN/ifengine/leptos/src/consts.rs) | `200ms` | Page fade-in transition duration. |
