use ifengine::view::Object;

#[ifengine::ifview]
fn test_element_macro_page(_: &mut ()) {
    use ifengine::elements::{l, link, ps, s};
    let span = s!("span_x").cls("fade").style("color", "blue");
    let link_span = link!("link_text");
    let line = l!(span, link_span).cls("my-line");
    ps!(line);
}

#[test]
fn test_element_macros_and_keys() {
    let mut game = ifengine::Game::new_with_page("test_element_macro_page", test_element_macro_page);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 1);
    if let Object::Paragraph(line) = &view.inner[0] {
        let line_id = line.id.expect("line must have an element id");
        assert_eq!(line_id >> 48, 1);
        assert_eq!(line.classes, vec!["my-line"]);

        assert_eq!(line.spans.len(), 2);
        let s0 = &line.spans[0];
        let s1 = &line.spans[1];

        let id0 = s0.id.expect("s0 must have an element id");
        let id1 = s1.id.expect("s1 must have an element id");

        assert_eq!(id0 >> 48, 1);
        assert_eq!(id1 >> 48, 1);

        // Distinct keys (different columns)
        assert_ne!(id0, id1);

        // Classes and styles
        assert_eq!(s0.classes, vec!["fade"]);
        assert_eq!(s0.style.get("color").map(|s| s.as_str()), Some("blue"));
    } else {
        panic!("expected Object::Paragraph");
    }
}
