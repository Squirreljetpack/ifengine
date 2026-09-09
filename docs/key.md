# Page State Keys in `ifengine`

In `ifengine`, interactive elements (`click!`, `choice!`, `dchoice!`, `alts!`, `count!`, `mchoice!`, `dparagraph!`) maintain their state across renders using a 64-bit integer called a **`PageKey`** (`u64`).

Each page maintains a persistent key-value store (`PageMap`) that maps `PageKey -> u64`. When a user interacts with a page (e.g. clicking a link or choice), the engine updates the value associated with the element's key and triggers a re-render.

---

## 1. 64-Bit Segmented Layout

Keys are divided into four 16-bit segments (`u16`), providing clear domain separation, human-readable hex inspection in debuggers, and collision-free loop support:

```text
 63          48 47          32 31          16 15           0
+--------------+--------------+--------------+--------------+
|   Counter    |  File Hash   | Line Number  | Column Number|
|   (16 bits)  |  (16 bits)   |  (16 bits)   |  (16 bits)   |
+--------------+--------------+--------------+--------------+
```

### Domain Separation via Top 16 Bits:
- **User Manual Keys (`Counter == 0`):**
  - The top 16 bits are always `0x0000`.
  - Leaves the remaining 48 bits (`0x0000_FFFF_FFFF_FFFF`) for user key payloads.
  - Normal integer literals like `0`, `1`, `6`, `42`, etc. naturally have their top 16 bits as `0`.
  - Manual keys can be read directly via `read_key!(6)` without requiring any normalization or masking translation.

- **Auto-Generated Keys (`Counter >= 1`):**
  - The top 16 bits store a 1-based instance counter (`1..=65535`).
  - First invocation at a call-site receives `Counter = 1`.
  - Subsequent invocations at the same call-site (e.g. inside a loop) receive `Counter = 2, 3, ...`.

---

## 2. Compile-Time Location Payload (Lower 48 Bits)

When a macro auto-generates a key, it builds a 48-bit location payload:

1. **File Hash (Bits 47..32, 16 bits):**
   Computed at compile-time using a `const` FNV-1a hash of `file!()`
2. **Line Number (Bits 31..16, 16 bits):**
3. **Column Number (Bits 15..0, 16 bits):**

Because `line` and `column` occupy dedicated bit positions, two different elements in the same file will never collide.

---

## 3. Runtime Loop Tracking

Because page rendering is deterministic, iteration `0`, `1`, `2`, ... will receive the exact same sequence of keys on every render:

```rust
let mut choices = vec![];
for i in 0..3 {
    // Each iteration automatically receives a distinct, stable key!
    // Iteration 0: 0x0001_[file]_[line]_[col]
    // Iteration 1: 0x0002_[file]_[line]_[col]
    // Iteration 2: 0x0003_[file]_[line]_[col]
    let e = click!(names[i * 2], {
        state.myname = names[i * 2].to_string();
        NEXT!(p4);
    });
    choices.push([e, s!(".  ", names[i * 2 + 1])]);
}
dchoice!(choices);
```

---

## 4. Manual Key Overrides

You can manually specify a key for an element by passing an expression in parentheses as the first argument:

```rust
// Assign manual key 6:
let span = count!((6), |val| format!("Clicked {} times", val));

// Read the value elsewhere on the page:
let clicks = read_key!(6).unwrap_or(0);
```

### Syntax Across Macros
Almost all interactive macros accept an optional `(key)` prefix:
- `click!((key), "text", { ... })`
- `count!((key), |n| ...)`
- `choice!((key), ...)`
- `dchoice!((key), choices, ...)`
- `alts!((key), variant?, [...])`
- `mchoice!((key), ...)`
- `dparagraph!((key), "text with [[link]]")`
- `mparagraph!((key), "text with [[link]]")`
- `replace!((key), "text with [[link]]", { ... })`

When a manual key is passed, the macro masks it to the lower 48 bits (`expr as u64 & 0x0000_FFFF_FFFF_FFFF`), guaranteeing it stays in the user domain.

---

## 5. Structural Element Keys & View Transitions

In addition to interactive state storage, element keys participate in frontend UI transitions (such as CSS View Transitions in Leptos: `style="view-transition-name: item-{id}"`).

### Top-Level Object Stamping (`StampedObject`)

Every element pushed to a `View` is wrapped in a **`StampedObject { id: Option<PageKey>, object: Object }`**:
- **Automatic Object Stamping**: All macros emitting top-level view objects (`p!`, `ps!`, `text!`, `texts!`, `h!`, `hr!`, `img!`, `choice!`, `mchoice!`, `dchoice!`, `replace!`, `EMBED!`) automatically stamp the outer `StampedObject` with an `auto_key()` (or an explicit author-specified `(key)`).
- **Container-Level Transitions**: Frontends attach `view-transition-name: item-{id}` directly to the outer container element (`<p>`, `<div>`, `<section>`). This guarantees smooth geometric interpolation (reflow/sliding) across state changes for all content blocks on the page.
- **Choice Morphing**: When a `choice!` is selected, its replacement paragraph inherits the choice's `PageKey`. The frontend pairs the old choice container snapshot with the new paragraph snapshot, morphing the choice block smoothly into text.

### Inline Elements (`Span` & `Line`)

Within composite objects, individual inline spans and lines can carry their own item-level `PageKey`s:
- **Interactive Spans**: `alts!`, `count!`, `click!`, `link!`, `tun!`, `back!`, and `s!` stamp returned spans with an ID, allowing in-place crossfade or cycle animations without reflowing the containing paragraph.
- **Composite Lines**: `l!` creates a `Line` with an explicit key for line-level transitions.

---

## 6. Key Management Helpers

The following macros and methods are available for working directly with keys:

### Macros
| Macro | Description |
| :--- | :--- |
| `read_key!(key)` | Reads the raw `u64` value associated with `key` from the current page's state. |
| `read_key_mask!(key [, N])` | Reads `key` as a bitmask, returning a `[bool; N]` array (defaults to `N = 64`). |
| `set_key!(key, value)` | Inserts or updates the `u64` value for `key` in page state. |
