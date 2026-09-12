# Element Margins & Spacing in `ifengine`

This document details the vertical rhythm, spacing, and margin rules used across all view elements in both frontends: **Leptos** (web / DOM) and **egui** (desktop / immediate-mode GUI).

---

## 1. Quick Comparison Reference

| Element / Macro | Variant | Leptos (Web) | egui (Desktop/GUI) |
| :--- | :--- | :--- | :--- |
| `p!`, `ps!`, `paragraph!`, `paragraphs!` | [`Object::Paragraph`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L23-L24) | `margin: 1.35rem 0;`<br>(collapses with adjacent margins; carries `RenderData`: `m-{val}` overrides inline margin, e.g. `m-0` -> `margin: 0;`; `:{speaker}` renders as dialogue with speaker marker; `"popup"`/`"modal"` opens modal overlay) | `ui.draw_empty(1)` (~18px / 1 line height) before (if not first) and after |
| `h1!` – `h6!` | [`Object::Heading`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L31-L32) | `margin-top: 1.8rem;`<br>`margin-bottom: 0.8rem;` | Symmetric `add_space(margin)` above and below based on heading level (4px – 20px) |
| `choice!`, `dchoice!`, `mchoice!` | [`Object::Choice`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L27-L28) | `margin: var(--passage-gap, 1.35rem) 0;`<br>`gap: 0.6rem;` between items<br>(collapses with adjacent margins) | `ui.draw_empty(1)` before and after;<br>`item_spacing.y: 10.0px` between choice items |
| `img!` | [`Object::Image`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L29-L30) | `margin: var(--passage-gap, 1.35rem) 0;` | Default widget spacing |
| `break!` | [`Object::Break`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L33-L34) | `margin: 2.25rem 0;` | Default `egui::Separator` spacing |
| `empty!(n)` | [`Object::Empty`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L35-L36) | `height: {n * 1.5}em;`<br>(no outer margin) | `ui.draw_empty(n)` (`n * 18px` / row height) |
| `note!` | [`Object::Note`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L40-L42) | `margin: 1rem 0;`<br>`padding: 0.75rem 1.25rem;` | *(Pending egui implementation)* |
| `EMBED!(page)` | [`Object::Embed`](file:///Users/absinthe/gh/OWN/ifengine/ifengine_core/src/view/mod.rs#L45-L48) | Standard: `margin: 0;` (seamless collapse with inner elements)<br>Modal: `margin: 0;` backdrop, `padding: 2rem;` | Recursively dispatches embedded view elements directly |

---

## 2. Element Specifications

### Paragraphs (`p!`, `ps!`, `paragraph!`, `paragraphs!`)
* **Variant**: `Object::Paragraph(Line, RenderData)`
* **Intended Use**: Main narrative prose, dialogue, character speech, and styled/compact text blocks.
* **Trailer Syntax**: Supports optional trailing `:: "metadata"` to set `RenderData` (defaults to `""`).
* **RenderData Conventions**:
  - `":speaker"`: Styled as dialogue in Leptos (`.passage-dialogue`), showing a speaker marker label (`.dialogue-speaker`) on the left instead of a box.
  - `"m-{val}"` (e.g. `"m-0"`, `"m-0.0"`, `"m-1.5"`): Overrides the default margin. `0` or `0.0` sets `margin: 0;`, non-zero numeric sets `margin: {val}rem;`, and unit strings (e.g. `m-10px`) set `margin: 10px;`.
  - `"popup"` or `"modal"`: Renders in a fixed modal backdrop overlay (`.passage-popup-backdrop` and `.passage-popup-dialog`).
* **Leptos**:
  - Class: `.passage-paragraph` (plus `.passage-dialogue` if `render_data` starts with `':'`)
  - CSS: Default `margin: 1.35rem 0; word-break: break-word;` (collapses with adjacent elements; overridden by `m-{}` or custom classes).
  - **Auto-collapse**: Margins collapse to `0 !important` if the child line is empty (`:has(> .passage-line:empty)`), or while awaiting entrance delay (`.delayed-hidden`) or exit removal (`.fade-out-removed`).
* **egui**:
  - If not the first element in the view, adds `ui.draw_empty(1)` before the line.
  - Adds `ui.draw_empty(1)` immediately after the line.
  - `draw_empty(1)` computes `TextStyle::Body` height (default `18.0px`) and calls `ui.add_space(row_height)`.

---

### Headings (`h1!` through `h6!`)
* **Variant**: `Object::Heading(Span, level)` (levels 1–6)
* **Intended Use**: Section titles, scene titles, and subtitles.
* **Leptos**:
  - Class: `.passage-heading.h{level}`
  - CSS: `margin-top: 1.8rem; margin-bottom: 0.8rem; line-height: 1.3;`
  - **Proximity Rule**: `.passage-heading + .passage-paragraph, .passage-heading + .passage-heading { margin-top: 0.6rem; }` gives tighter visual binding to immediately following subtitles or lead text.
  - Type scales:
    - `h1`: `2.1rem`
    - `h2`: `1.7rem`
    - `h3`: `1.4rem` (`letter-spacing: 0.08em; text-transform: uppercase;`)
    - `h4`: `1.2rem`
    - `h5`: `1.05rem`
    - `h6`: `0.95rem` (`text-transform: uppercase; color: var(--color-text-dim);`)
* **egui**:
  - Method: `ui.draw_heading(rich_text, level)` in [`egui/src/utils.rs`](file:///Users/absinthe/gh/OWN/ifengine/egui/src/utils.rs#L13-L34).
  - Inserts symmetric vertical spacing before and after the label via `ui.add_space(margin)`:
    - Level 0: Font `144.0pt`, margin `20.0px`
    - Level 1: Font `96.0pt`, margin `16.0px`
    - Level 2: Font `64.0pt`, margin `12.0px`
    - Level 3: Font `48.0pt`, margin `8.0px`
    - Level 4: Font `32.0pt`, margin `6.0px`
    - Level 5: Font `26.0pt`, margin `5.0px`
    - Level 6+: Font `22.0pt`, margin `4.0px`

---

### Choices (`choice!`, `dchoice!`, `mchoice!`)
* **Variant**: `Object::Choice(Vec<(u8, Line)>)`
* **Intended Use**: Interactive branch menus and player option selections.
* **Leptos**:
  - Container Class: `.passage-choices.choice-container`
  - Container CSS: `margin: var(--passage-gap, 1.35rem) 0; display: flex; flex-direction: column; gap: 0.6rem;` (collapses with adjacent elements)
  - Auto-collapses (`display: none !important; margin: 0 !important; padding: 0 !important;`) if child choices are hidden/delayed.
* **egui**:
  - Wrapped in a vertical layout:
    ```rust
    ui.draw_empty(1);
    ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);
    // ... choices ...
    ui.draw_empty(1);
    ```
  - Top & bottom spacing: 1 empty line height (~18px).
  - Spacing between options: `10.0px`.

---

### Images (`img!`)
* **Variant**: `Object::Image(Image)`
* **Intended Use**: Inline and full-width illustrations or banners.
* **Leptos**:
  - Container Class: `.passage-image-wrapper`
  - CSS: `margin: var(--passage-gap, 1.35rem) 0; text-align: center;`
  - Image element: `max-width: 100%; height: auto; border-radius: 4px;`
* **egui**:
  - Added directly to the UI layout using dimensions `(w, h)` with default widget spacing.

---

### Section Breaks (`break!`)
* **Variant**: `Object::Break`
* **Intended Use**: Scene or thematic transitions within a page.
* **Leptos**:
  - Class: `<hr class="passage-break">`
  - CSS: `border: none; text-align: center; margin: 2.25rem 0;`
  - Content: Renders triple asterisks (`*  *  *`) via `::after` with `letter-spacing: 0.4em`.
* **egui**:
  - Renders `egui::Separator::default()`.

---

### Vertical Empty Spacers (`empty!(n)`)
* **Variant**: `Object::Empty(u8)`
* **Intended Use**: Explicit vertical white space.
* **Leptos**:
  - Class: `.passage-empty`
  - Style: Dynamic inline height `style="height: {n * 1.5}em;"` (`1.5em` per unit `n`).
  - Margins: None.
* **egui**:
  - Calls `ui.draw_empty(n)`, which adds `row_height * n as f32` pixels (where `row_height` is ~18px).

---

### Notes (`note!`)
* **Variant**: `Object::Note(Line, (u8, u8))`
* **Intended Use**: Footnotes, marginalia, or explanatory callouts.
* **Leptos**:
  - Class: `<aside class="passage-note">`
  - CSS:
    - `margin: 1rem 0;`
    - `padding: 0.75rem 1.25rem;`
    - `border: 1px dashed var(--color-border);`

---

### Embedded Views & Modals (`EMBED!`)
* **Variant**: `Object::Embed(View, RenderData)`
* **Intended Use**: Nested subpages, inventory drawers, or modal/popup dialogs.
* **Leptos**:
  - **Standard Inline Embed**:
    - Class: `.passage-embed`
    - CSS: `display: block; margin: 0;` (transparent flow allowing inner elements to collapse with outer elements)
  - **Popup / Modal (`render_data == "popup" | "modal"`)**:
    - Backdrop (`.passage-popup-backdrop`): `position: fixed; inset: 0; margin: 0 !important;`
    - Dialog (`.passage-popup-dialog`): `padding: 2rem; row-gap: 1.25rem; max-width: min(90vw, var(--page-width));`
* **egui**:
  - Recursively evaluates and renders the inner sub-view within the existing layout container.

---

## 3. Outer Container Margins & Padding

### Leptos Outer Shell
* `#page`:
  - `padding: 2rem 2rem 1rem;`
  - `row-gap: 1.5rem;`
  - `max-width: var(--page-width, 44rem);`
* `.passage-article`:
  - `display: flow-root;` (establishes an independent Block Formatting Context for native CSS vertical margin collapse between children)
  - `margin-top: auto; margin-bottom: auto;` (vertically centered reading canvas within `#page` flex column)
* `.passage-header`:
  - `margin-bottom: 0;` (when content present, `padding-bottom: 0.5rem; border-bottom: 1px solid var(--color-border);`)
* `.passage-footer`:
  - `margin-top: 0; padding-top: 0.5rem; padding-bottom: 0.25rem;`

### egui Outer Shell
* Main central window / panel margins:
  - Panel inner margin: `egui::Margin::symmetric(5, 12)`
  - Reading column outer margin: centered horizontal column
  - Default item spacing: `ui.spacing().item_spacing` (typically `(8.0, 4.0)` unless overridden in choice vertical layout)
