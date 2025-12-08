weave: inkle threading: weave!(widget1, widget2...) {
	follow each with a game clone until a view is reached, store the widget_idx, action_idx (using get_interactables?), choice_value
}
text input element: PageMap should be (HashMap, StringMap)


egui graph
 - layout
 - single edge for backlinks
 - etc
Full render, doc and macro support for all object types

- derive aliases may be cool


# EGUI
- Smaller size
    maybe configure trunk: wasm-pack build -t web --release \
    --manifest-path ./Cargo.toml \
    -Z build-std=panic_abort,std -Z build-std-features=panic_immediate_abort
    maybe other frameworks will allow lazy loading
- somehow the footer underline doesn't get applied in trunk build sometimes despite ok in in trunk serve
- Fade transitions
    - we could probably fade outgoing to nothing with animate_bool, then fade in the next page but dunno if there's a cleaner crate approach.
- An actual handcrafted theme for default
- Finished for now, some web fw like leptos is probably a better fit
- header panel broken on mac for some reason

# Docs
- cannot link docs from proc macro until crate split (could also use absolute urls)
- MaybeKey is private

# Note
passing data between pages?
- all thru state/tags good enough?
maybe dbg formatting could be improved, i.e. bitmask values, string keys, whatever

Do we want to replace "''" with "'"?
