use ifengine::elements::{EMBED, choice, extend, p, s, text, x};
use ifengine::view::Object;

#[derive(Debug, Default, Clone, PartialEq)]
struct GameState {
    pub value: usize,
}

#[ifengine::ifview]
fn page_paragraph_extend(_: &mut GameState) {
    let name = "Adventurer";
    let hp = 75;
    let max_hp = 100;
    p!("Hello,");
    x!(" {name}!");
    x!("span": s!(" Welcome to the dungeon."));
    x!(" (HP: {hp}/{max_hp})");
    extend!(" Good luck!");
}

#[test]
fn test_paragraph_extend() {
    let mut game = ifengine::Game::new_with_page("page_paragraph_extend", page_paragraph_extend);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 1);
    if let Object::Paragraph(line) = &view.inner[0].object {
        assert_eq!(
            line.content(),
            "Hello, Adventurer! Welcome to the dungeon. (HP: 75/100) Good luck!"
        );
        assert!(line.spans.len() >= 6);
    } else {
        panic!("expected Object::Paragraph");
    }
}

#[ifengine::ifview]
fn page_text_extend(_: &mut GameState) {
    text!("Initial");
    x!("span": " text", " line");
}

#[test]
fn test_text_extend() {
    let mut game = ifengine::Game::new_with_page("page_text_extend", page_text_extend);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 1);
    if let Object::Text(line, _) = &view.inner[0].object {
        assert_eq!(line.content(), "Initial text line");
        assert_eq!(line.spans.len(), 3);
    } else {
        panic!("expected Object::Text");
    }
}

#[ifengine::ifview]
fn page_choice_extend(_: &mut GameState) {
    choice! {
        "Option 0" => "Chose 0",
        "Option 1" => "Chose 1",
    };
    // Explicit numbering
    x!("choice": (2, "Option 2"), (3, "Option 3"));
    // Auto-numbering via into_numbered_line (Option<u8> is None => previous + 1)
    x!("choice": "Option 4", "Option 5");
    // Also works with unquoted ident prefix `choice:`
    x!(choice: (10, "Option 10"), "Option 11");
}

#[test]
fn test_choice_extend() {
    let mut game = ifengine::Game::new_with_page("page_choice_extend", page_choice_extend);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 1);
    if let Object::Choice(choices) = &view.inner[0].object {
        assert_eq!(choices.len(), 8);
        assert_eq!(choices[0].0, 0);
        assert_eq!(choices[0].1.content(), "Option 0");
        assert_eq!(choices[1].0, 1);
        assert_eq!(choices[1].1.content(), "Option 1");
        assert_eq!(choices[2].0, 2);
        assert_eq!(choices[2].1.content(), "Option 2");
        assert_eq!(choices[3].0, 3);
        assert_eq!(choices[3].1.content(), "Option 3");
        // Auto-incremented from 3: 4 and 5
        assert_eq!(choices[4].0, 4);
        assert_eq!(choices[4].1.content(), "Option 4");
        assert_eq!(choices[5].0, 5);
        assert_eq!(choices[5].1.content(), "Option 5");
        // Explicit 10, then auto-incremented to 11
        assert_eq!(choices[6].0, 10);
        assert_eq!(choices[6].1.content(), "Option 10");
        assert_eq!(choices[7].0, 11);
        assert_eq!(choices[7].1.content(), "Option 11");
    } else {
        panic!("expected Object::Choice");
    }
}

#[ifengine::ifview]
fn subpage_for_embed(_: &mut GameState) {
    p!("Embedded paragraph");
}

#[ifengine::ifview]
fn page_embed_extend(_: &mut GameState) {
    EMBED!(subpage_for_embed);
    x!("object": Object::Break);
    x!(object: Object::Break);
}

#[test]
fn test_embed_extend() {
    let mut game = ifengine::Game::new_with_page("page_embed_extend", page_embed_extend);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 1);
    if let Object::Embed(sub_view, _) = &view.inner[0].object {
        assert_eq!(sub_view.inner.len(), 3);
        assert!(matches!(sub_view.inner[0].object, Object::Paragraph(_)));
        assert!(matches!(sub_view.inner[1].object, Object::Break));
        assert!(matches!(sub_view.inner[2].object, Object::Break));
    } else {
        panic!("expected Object::Embed");
    }
}

#[ifengine::ifview]
fn page_shape_mismatch(_: &mut GameState) {
    p!("Just a paragraph");
    // Choice shape pushed onto paragraph should do nothing
    x!("choice": (1, "Should not be added"));
    x!("object": Object::Break);
}

#[test]
fn test_shape_mismatch_does_nothing() {
    let mut game = ifengine::Game::new_with_page("page_shape_mismatch", page_shape_mismatch);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 1);
    if let Object::Paragraph(line) = &view.inner[0].object {
        assert_eq!(line.content(), "Just a paragraph");
        assert_eq!(line.spans.len(), 1);
    } else {
        panic!("expected Object::Paragraph");
    }
}

#[ifengine::ifview]
fn page_empty_view_extend(_: &mut GameState) {
    // When view is empty, extend should safely do nothing
    x!("No previous object");
    x!("choice": "No previous choice");
    x!("object": Object::Break);
}

#[test]
fn test_empty_view_extend_does_nothing() {
    let mut game = ifengine::Game::new_with_page("page_empty_view_extend", page_empty_view_extend);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 0);
}
