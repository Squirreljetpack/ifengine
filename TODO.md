weave: inkle threading: weave!(widget1, widget2...) {
	follow each with a game clone until a view is reached, store the widget_idx, action_idx (using get_interactables?), choice_value
}
text input element: PageMap should be (HashMap, StringMap)


egui graph
 - layout
 - single edge for backlinks
 - etc
Full render, doc and macro support for all object types

- change hide_if to hide_in_sim
- derive aliases may be cool

# EGUI
- reduce binary size (currently 15mb)
    - reduced to 7
    - is dynamic font loading possible?
- maybe configure trunk: wasm-pack build -t web --release \
    --manifest-path ./Cargo.toml \
    -Z build-std=panic_abort,std -Z build-std-features=panic_immediate_abort
maybe other frameworks will allow lazy loading
- somehow the footer underline doesn't get applied in trunk build despite ok in in trunk serve
- `/#graph` should show graph
- flickering: transitions should help
- An actual handcrafted theme for default
- Finished for now, some web fw like leptos is probably a better fit

# Docs
- cannot link docs from proc macro until crate split
- MaybeKey is private

# Note
tracking box dyn any can pass data between pages more conveniently but its probably a bad idea
maybe dbg formatting could be improved, i.e. bitmask values, string keys, whatever

Do we want to replace "''" with "'"?
