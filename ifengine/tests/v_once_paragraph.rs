use ifengine::Game;
use ifengine::view::Object;

#[derive(Clone, Debug, Default)]
struct VisitedState {
    present: bool,
    absent: bool,
}

#[ifengine::ifview]
fn page_visited(state: &mut VisitedState) {
    ifengine::elements::set!("seen");
    state.present = ifengine::v!("seen");
    state.absent = ifengine::v!("missing");
}

#[test]
fn test_v_is_some_alias() {
    let mut game = Game::new_with_page("page_visited", page_visited);
    let _ = game.view().expect("view should succeed");

    assert!(game.context.present, "v! is true for a set key");
    assert!(!game.context.absent, "v! is false for a missing key");
}

#[derive(Clone, Debug, Default)]
struct OnceState {
    runs: usize,
}

#[ifengine::ifview]
fn page_once(state: &mut OnceState) {
    ifengine::elements::once! {
        state.runs += 1;
    }
}

#[test]
fn test_once_runs_on_first_render_only() {
    let mut game = Game::new_with_page("page_once", page_once);
    let _ = game.view().expect("view should succeed");
    assert_eq!(game.context.runs, 1);

    let _ = game.view().expect("view should succeed");
    let _ = game.view().expect("view should succeed");
    assert_eq!(game.context.runs, 1, "once! body must not run again");
}

#[derive(Clone, Debug, Default)]
struct FreshState {
    runs: usize,
}

#[ifengine::ifview]
fn page_fresh_a(state: &mut FreshState) {
    ifengine::elements::once!(0: {
        state.runs += 1;
    });
    ifengine::elements::p!(ifengine::elements::link!("to b", page_fresh_b));
}

#[ifengine::ifview]
fn page_fresh_b(_: &mut FreshState) {
    ifengine::elements::p!(ifengine::elements::link!("back", page_fresh_a));
}

fn first_action(view: &ifengine::View) -> ifengine::core::Actions {
    let Object::Paragraph(line, _) = &view.inner[0].object else {
        panic!("expected a paragraph");
    };
    line.spans
        .iter()
        .find(|span| span.active())
        .expect("expected an active link span")
        .action
        .clone()
}

#[test]
fn test_once_zero_runs_on_every_fresh_load() {
    let mut game = Game::new_with_page("page_fresh_a", page_fresh_a);

    let view = game.view().expect("view should succeed");
    assert_eq!(game.context.runs, 1);

    game.handle_action(first_action(&view)).expect("action");
    let view = game.view().expect("view should succeed");
    assert_eq!(game.context.runs, 1, "leaving the page does not re-run it");

    game.handle_action(first_action(&view)).expect("action");
    let _ = game.view().expect("view should succeed");
    assert_eq!(game.context.runs, 2, "returning to the page runs once!(0:)");
}

#[ifengine::ifview]
fn page_bracket_escape(_: &mut ()) {
    ifengine::elements::p!("Send [[brackets]] literally, click [link] instead");
}

#[test]
fn test_double_bracket_is_literal() {
    let mut game = Game::new_with_page("page_bracket_escape", page_bracket_escape);
    let view = game.view().expect("view should succeed");

    let Object::Paragraph(line, _) = &view.inner[0].object else {
        panic!("expected a paragraph");
    };

    assert_eq!(
        line.content(),
        "Send [brackets] literally, click link instead"
    );
    let link = line
        .spans
        .iter()
        .find(|span| span.content == "link")
        .expect("expected the [link] span");
    assert!(link.active(), "[link] must be interactive");
    assert_eq!(line.spans[0].content, "Send [brackets] literally, click ");
}

#[derive(Clone, Debug, Default)]
struct ParagraphKeyState {
    visited: bool,
    target: String,
}

#[ifengine::ifview]
fn page_paragraph_string_key(state: &mut ParagraphKeyState) {
    use ifengine::elements::{p, v};
    state.target = p!(("link_key"), "Click [here]").unwrap_or_default();
    state.visited = v!("link_key");
}

#[test]
fn test_paragraph_explicit_string_key_is_observable() {
    let mut game = Game::new_with_page("page_paragraph_string_key", page_paragraph_string_key);
    let view = game.view().expect("view should succeed");
    assert!(!game.context.visited);

    let Object::Paragraph(line, _) = &view.inner[0].object else {
        panic!("expected a paragraph");
    };
    let action = line
        .spans
        .iter()
        .find(|span| span.content == "here")
        .expect("expected the [here] link")
        .action
        .clone();

    game.handle_action(action).expect("action");
    let _ = game.view().expect("view should succeed");

    assert_eq!(game.context.target, "here");
    assert!(
        game.context.visited,
        "v!(key) sees the paragraph's dirty mark"
    );
}

#[derive(Clone, Debug, Default)]
struct SharedLinkState {
    first: String,
    second: String,
}

#[ifengine::ifview]
fn page_shared_link(state: &mut SharedLinkState) {
    use ifengine::elements::p;
    state.first = p!("First [more]").unwrap_or_default();
    state.second = p!("Second [more]").unwrap_or_default();
}

#[test]
fn test_shared_link_text_stays_with_own_paragraph() {
    let mut game = Game::new_with_page("page_shared_link", page_shared_link);
    let view = game.view().expect("view should succeed");

    let Object::Paragraph(second_line, _) = &view.inner[1].object else {
        panic!("expected a paragraph");
    };
    let action = second_line
        .spans
        .iter()
        .find(|span| span.content == "more")
        .expect("expected the [more] link")
        .action
        .clone();

    game.handle_action(action).expect("action");
    let _ = game.view().expect("view should succeed");

    assert_eq!(game.context.second, "more");
    assert_eq!(
        game.context.first, "",
        "the first paragraph must not steal the click"
    );
}
