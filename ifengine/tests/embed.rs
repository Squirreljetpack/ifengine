use ifengine::elements::{EMBED, NEXT, p};
use ifengine::view::Object;

#[derive(Debug, Default, Clone, PartialEq)]
struct TestState {
    pub counter: usize,
    pub log: Vec<String>,
}

#[ifengine::ifview]
fn subpage_view(s: &mut TestState) {
    s.counter += 10;
    s.log.push("subpage_called".into());
    p!("Embedded subpage content");
}

#[ifengine::ifview]
fn parent_page(s: &mut TestState) {
    p!("Parent header");
    s.counter += 1;
    EMBED!(subpage_view);
    s.counter += 2;
    p!("Parent footer");
}

#[test]
fn test_embed_view_and_context_mutation() {
    let mut game = ifengine::Game::new_with_page("parent_page", parent_page);
    game.context = TestState::default();

    let view = game.view().expect("view should succeed");

    // Check parent view structure: Paragraph, Embed, Paragraph
    assert_eq!(view.inner.len(), 3);
    assert!(matches!(view.inner[0].object, Object::Paragraph(_)));
    if let Object::Embed(sub_view, _) = &view.inner[1].object {
        assert_eq!(sub_view.inner.len(), 1);
        assert!(sub_view.pageid.0.ends_with("subpage_view"));
        assert_ne!(sub_view.pageid, view.pageid);
    } else {
        panic!("expected Object::Embed at index 1");
    }
    assert!(matches!(view.inner[2].object, Object::Paragraph(_)));

    // Check context mutations: 1 (parent before) + 10 (subpage) + 2 (parent after) = 13
    assert_eq!(game.context.counter, 13);
    assert_eq!(game.context.log, vec!["subpage_called"]);
}

#[ifengine::ifview]
fn target_destination(_: &mut TestState) {
    p!("Destination page arrived!");
}

#[ifengine::ifview]
fn subpage_transition(_: &mut TestState) {
    NEXT!(target_destination);
}

#[ifengine::ifview]
fn parent_with_transition_embed(s: &mut TestState) {
    p!("Before embed transition");
    s.counter = 999;
    EMBED!(subpage_transition);
    // This line should NOT be reached because EMBED immediately early-returns other Response variants!
    s.counter = 0;
    p!("Should never appear");
}

#[test]
fn test_embed_propagates_transitions() {
    let mut game =
        ifengine::Game::new_with_page("parent_with_transition_embed", parent_with_transition_embed);
    game.context = TestState::default();

    let view = game.view().expect("view should succeed");

    // View should have transitioned to target_destination!
    assert_eq!(view.inner.len(), 1);
    if let Object::Paragraph(line) = &view.inner[0].object {
        assert_eq!(line.content(), "Destination page arrived!");
    } else {
        panic!("expected destination paragraph");
    }

    // Context should have counter = 999 (was not overwritten with 0)
    assert_eq!(game.context.counter, 999);
}

#[ifengine::ifview]
fn grandchild(s: &mut TestState) {
    s.counter += 100;
    p!("Grandchild content");
}

#[ifengine::ifview]
fn child_page(s: &mut TestState) {
    p!("Child content");
    EMBED!(grandchild);
}

#[ifengine::ifview]
fn root_page(s: &mut TestState) {
    p!("Root content");
    EMBED!(child_page);
}

#[test]
fn test_nested_embeds() {
    let mut game = ifengine::Game::new_with_page("root_page", root_page);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 2);
    if let Object::Embed(child_view, _) = &view.inner[1].object {
        assert_eq!(child_view.inner.len(), 2);
        if let Object::Embed(grandchild_view, _) = &child_view.inner[1].object {
            assert_eq!(grandchild_view.inner.len(), 1);
        } else {
            panic!("expected nested Object::Embed for grandchild");
        }
    } else {
        panic!("expected Object::Embed for child");
    }

    assert_eq!(game.context.counter, 100);
}

#[test]
fn test_reverse_page_id_lookup() {
    let canonical = ifengine::core::resolve_page_id(subpage_view);
    assert_eq!(canonical, Some("embed::subpage_view"));
}

#[ifengine::ifview]
fn embedded_interactable_child(s: &mut TestState) {
    ifengine::elements::choice! {
        "Child Option A" => {
            s.counter += 50;
            "Selected A"
        },
        "Child Option B" => {
            s.counter += 100;
            "Selected B"
        },
    };
}

#[ifengine::ifview]
fn parent_with_embed_interactables(_s: &mut TestState) {
    p!("Parent header");
    EMBED!(embedded_interactable_child);
    p!("Parent footer");
}

#[test]
fn test_embed_expanded_in_interactables() {
    let mut game = ifengine::Game::new_with_page(
        "parent_with_embed_interactables",
        parent_with_embed_interactables,
    );
    let view = game.view().expect("view should succeed");

    // interactables() and interactables_sim() must directly surface the child's choices
    let sim_interactables = view.interactables_sim();
    assert_eq!(sim_interactables.len(), 2);
    assert_eq!(sim_interactables[0].content(), "Child Option A");
    assert_eq!(sim_interactables[1].content(), "Child Option B");

    // Interact with the first choice from the parent view
    game.interact(sim_interactables[0]).expect("interact succeeded");
    let _next_view = game.view().expect("view after choice");
    assert_eq!(game.context.counter, 50);
}

