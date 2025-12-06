inkle threading: show!(widget1, widget2...) {
	follow each with a game clone until a view is reached, store the widget_idx, choice_idx, choice_value
}
text input element: PageMap should be (HashMap, StringMap)


egui graph draw
egui widget bar

bring in local dependencies to enable github pages build

# EGUI
reduce binary size (currently 15mb)
- maybe cofnigrue trunk: wasm-pack build -t web --release \
    --manifest-path ./Cargo.toml \
    -Z build-std=panic_abort,std -Z build-std-features=panic_immediate_abort
maybe other frameworks will allow lazy loading

somehow the footer underline doesn't get applied in trunk build despite ok in in trunk serve

- `/#graph` should show graph