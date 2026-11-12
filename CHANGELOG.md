## [0.2.0] - 2026-11-11

### 🚀 Features

- *(core)* `set!` and `get!` macros with `=>` link syntax, `once!`, `v!` visited
  pages and an enhanced `click!`
- *(core)* `alts!` variant prefixes, `tag!`/`untag!` with `Once:`/`Sticky:`
  prefixes, click masks on links, and multiple actions per span
- *(core)* paragraph macros build from lines and return `PData`; `paragraph!`
  replaces `dparagraph`, `dp!`/`mp!` are gone, and the trailer is now
  `:: [classes], "RenderData"`
- *(core)* in-place replacements in `l!`/`s!`, optional click limits on
  paragraphs, `Line::clear_actions`, and the `br!` spacer macro
- *(leptos)* transitions are driven by a Rust-built plan, with a dedicated
  scheduler and view-transition shim
- *(leptos)* clone-snapshot CSS backend for Gecko with FLIP reflow and in-place
  cross-fades, selected from engine capabilities or a `?vt` override
- *(leptos)* header actions, and a motion setting in a new options modal
- *(cli)* minimal CLI frontend and playtester

### 🐛 Bug Fixes

- *(leptos)* fix view-transition deadlocks, popping after fades, reflow tagging
  and secondary transition timings
- *(leptos)* classify transitions from the snapshot, keep header action
  callbacks alive across passages, and only mark delayed elements as seen once
  transitions run
- *(macros)* treat `max_clicks 0` as disabled and record content hash keys on
  replace links

### 📚 Documentation

- Align cheat sheet, macro docs, transition docs and architecture notes with the
  new macro set and transition backends

### ⚠️ Breaking Changes

- Macro syntax and link value layout changed; `dp!`/`mp!` are removed and
  `dparagraph` returns `PData` instead of an event string

## [0.1.2] - 2026-09-12

### 🚀 Features

- Extend!
- Trigger both action and choice when choice contains actions. Resolve mchoice RHS to lines.
- Remove InternalKey
- Support reentrant tunnels during simulation + other fixes

### 🐛 Bug Fixes

- Misc transitions

### 📚 Documentation

- Update

## [0.1.1] - 2026-09-09

### 🚀 Features

- *(leptos)* adjust styles
- *(core)* uniform stamping for objects
- *(core)* global pagehandle registry
- *(leptos)* save/load
- *(macros)* extend!

## [0.1.0] - 2026-09-09

### 🚀 Features

- Leptos fe
- EMBED
- Steal more from saltwrack for demo

### 📚 Documentation

- Cleanup

## [0.0.6] - 2026-05-15

### 🚀 Features

- Transitions
- Make initial menu visible, add iteration and depth counter to game state
- Scroll margin
- Fix unused fonts
- Various cleanup
- Rename some elements for clarity (link, back)

### 💼 Other

- Use egui_alignments from crates.io now that a compatible version has been published
