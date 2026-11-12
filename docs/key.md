# Page State Keys in `ifengine`

In `ifengine`, interactive elements (`click!`, `choice!`, `dchoice!`, `alts!`, `mchoice!`, `paragraph!`) maintain their state across renders using a 64-bit integer called a **`PageKey`** (`u64`).

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
  - Manual integer keys are read directly via `get!(6)` without any normalization or masking translation.
  - String keys are hashed with FNV-1a, which sets bit 63 to keep them out of the integer key space.

- **Auto-Generated Keys (`Counter >= 1`):**
  - Bits 48..62 store a 1-based instance counter (`1..=32767`); bit 63 is reserved for hashed keys.
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
let span = click!((6), |n| format!("Clicked {n} times"));

// Read the value elsewhere on the page:
let clicks = get!(6).unwrap_or(0);
```

### Syntax Across Macros
Almost all interactive macros accept an optional `(key)` prefix:
- `click!((key), "text", { ... })`
- `click!((key), |n| ...)`
- `choice!((key), ...)`
- `dchoice!((key), choices, ...)`
- `alts!((key), Variant: [...])`
- `mchoice!((key), ...)`
- `paragraph!((key), "text with [link]")`
- `replace!((key), "text with [link]" => replacement)`

When a manual key is passed, the macro converts it through `IntoPageKey`: integer keys are used as-is (their top 16 bits are `0`), while string keys are hashed.

---

## 5. Structural Element Keys & View Transitions

In addition to interactive state storage, element keys participate in frontend UI transitions (such as CSS View Transitions in Leptos: `style="view-transition-name: item-{id}"`).

### Top-Level Object Stamping (`StampedObject`)

Every element pushed to a `View` is wrapped in a **`StampedObject { id: Option<PageKey>, object: Object }`**:
- **Automatic Object Stamping**: All macros emitting top-level view objects (`p!`, `ps!`, `h!`, `hr!`, `img!`, `choice!`, `mchoice!`, `dchoice!`, `replace!`, `EMBED!`) automatically stamp the outer `StampedObject` with an `auto_key()` (or an explicit author-specified `(key)`).
- **Container-Level Transitions**: Frontends attach `view-transition-name: item-{id}` directly to the outer container element (`<p>`, `<div>`, `<section>`). This guarantees smooth geometric interpolation (reflow/sliding) across state changes for all content blocks on the page.
- **Choice Morphing**: When a `choice!` is selected, its replacement paragraph inherits the choice's `PageKey`. The frontend pairs the old choice container snapshot with the new paragraph snapshot, morphing the choice block smoothly into text.

### Inline Elements (`Span`)

Within composite objects, individual inline spans carry their own item-level `PageKey`s:
- **Interactive Spans**: `alts!`, `click!`, `link!`, `tun!`, `back!`, and `s!` stamp returned spans with an ID, allowing in-place crossfade or cycle animations without reflowing the containing paragraph.
- **Lines (`Line`)**: Purely inline groupings of spans with styling classes; they do not carry keys and inherit their container context.

---

## 6. Value Layout for Clickable Links

Links created by `paragraph!` (`[target]`) store their click state in the value of the link's own
key (`hash_key(target)`):

- Bits 0..=15: the click count (`LINK_COUNT_MASK`). Claiming a pending click increments it
  (saturating), so the count field saturates after 65535 clicks.
- Bits 16..=62: the location payload of the paragraph that owns the pending click, shifted up 16
  bits. The 48-bit payload window keeps 47 bits because the dirty marker takes its top bit.
- Bit 63: the *dirty* marker (`DIRTY_BIT`), set together with the location payload and cleared when
  the click is claimed.

`paragraph!` reads this layout directly: `PageState::poll_click(.., Some(key))` claims a pending click
only when the dirty marker is set and the location payload matches the paragraph, incrementing the
count field; a link whose count is non-zero no longer offers an action; and the location field plays
no part in counting — an explicit click limit simply sums the counts of the paragraph's distinct links.
`max_clicks: 0` disables every link immediately, while omitting the prefix leaves the paragraph
unbounded.

## 7. Key Management Helpers
### Macros
| Macro | Description |
| :--- | :--- |
| `get!(key)` | Reads the `Option<u64>` value associated with `key` from the current page's state. |
| `set!(key [, value])` | Inserts or updates the `u64` value for `key` (defaults to `0`). |
| `reset_key!(key)` | Removes `key` from page state. |
| `read_key_mask!(key [, N])` | Reads `key` as a bitmask, returning a `[bool; N]` array (defaults to `N = 64`). |
| `set_key_mask!(key, bits...)` / `unset_key_mask!(key, bits...)` | Sets or clears individual bits of `key`. |
