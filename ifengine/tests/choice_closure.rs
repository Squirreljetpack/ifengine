use ifengine::elements::{choice, replace, s};
use ifengine::view::Object;

#[derive(Clone, Debug, Default)]
struct TestState {
    flag: bool,
    value: String,
}

#[ifengine::ifview]
fn page_choice_closure_0(state: &mut TestState) {
    choice! {
        "Keep choice" => |l| {
            state.flag = true;
            l
        },
        "Extend choice with +" => |l| {
            l + " — now extended!"
        },
        "Extend choice with +=" => |mut l| {
            l += " — with add assign!";
            l
        },
        "Standard replacement" => {
            state.value = "changed".to_string();
            "Replaced completely"
        },
    };
}

#[test]
fn test_choice_closure_keep() {
    let mut game = ifengine::Game::new_with_page("page_choice_closure_0", page_choice_closure_0);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 1);
    let choice_key = view.inner[0].id.expect("choice key");
    let Object::Choice(choices) = &view.inner[0].object else {
        panic!("expected Object::Choice");
    };
    assert_eq!(choices.len(), 4);
    assert_eq!(choices[0].1.content(), "Keep choice");

    // Select choice 0: Keep choice
    game.inner.handle_choice(choice_key, 0);
    let view2 = game.view().expect("view should succeed");
    assert_eq!(view2.inner.len(), 1);
    let Object::Paragraph(line, _) = &view2.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(line.content(), "Keep choice");
    assert!(game.context.flag);
}

#[test]
fn test_choice_closure_extend_add() {
    let mut game = ifengine::Game::new_with_page("page_choice_closure_0", page_choice_closure_0);
    let view = game.view().expect("view should succeed");
    let choice_key = view.inner[0].id.expect("choice key");

    // Select choice 1: Extend choice with +
    game.inner.handle_choice(choice_key, 1);
    let view2 = game.view().expect("view should succeed");
    let Object::Paragraph(line, _) = &view2.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(line.content(), "Extend choice with + — now extended!");
}

#[test]
fn test_choice_closure_extend_add_assign() {
    let mut game = ifengine::Game::new_with_page("page_choice_closure_0", page_choice_closure_0);
    let view = game.view().expect("view should succeed");
    let choice_key = view.inner[0].id.expect("choice key");

    // Select choice 2: Extend choice with +=
    game.inner.handle_choice(choice_key, 2);
    let view2 = game.view().expect("view should succeed");
    let Object::Paragraph(line, _) = &view2.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(line.content(), "Extend choice with += — with add assign!");
}

#[test]
fn test_choice_closure_non_closure_unchanged() {
    let mut game = ifengine::Game::new_with_page("page_choice_closure_0", page_choice_closure_0);
    let view = game.view().expect("view should succeed");
    let choice_key = view.inner[0].id.expect("choice key");

    // Select choice 3: Standard replacement
    game.inner.handle_choice(choice_key, 3);
    let view2 = game.view().expect("view should succeed");
    let Object::Paragraph(line, _) = &view2.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(line.content(), "Replaced completely");
    assert_eq!(game.context.value, "changed");
}

#[ifengine::ifview]
fn page_choice_closure_multi_lhs(_: &mut TestState) {
    choice! {
        "North" | "South" => |l| l + " path taken",
    };
}

#[test]
fn test_choice_closure_multi_lhs() {
    let mut game = ifengine::Game::new_with_page("page_choice_closure_multi_lhs", page_choice_closure_multi_lhs);
    let view = game.view().expect("view should succeed");
    let choice_key = view.inner[0].id.expect("choice key");

    // Select choice 1: "South"
    game.inner.handle_choice(choice_key, 1);
    let view2 = game.view().expect("view should succeed");
    let Object::Paragraph(line, _) = &view2.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(line.content(), "South path taken");
}

#[ifengine::ifview]
fn page_choice_static_omitted_arrow(_: &mut TestState) {
    choice! {
        s!("Styled static choice").cls("in-500"),
        "Plain static choice",
    };
}

#[test]
fn test_choice_static_omitted_arrow_and_cleaned() {
    let mut game = ifengine::Game::new_with_page(
        "page_choice_static_omitted_arrow",
        page_choice_static_omitted_arrow,
    );
    let view = game.view().expect("view should succeed");
    let choice_key = view.inner[0].id.expect("choice key");

    // Initially, choice 0 has the "in-500" class on its span
    if let Object::Choice(choices) = &view.inner[0].object {
        assert_eq!(choices[0].1.spans[0].classes, vec!["in-500"]);
    } else {
        panic!("expected Object::Choice");
    }

    // Select choice 0: omitted arrow keeps choice text, and .clean() strips classes
    game.inner.handle_choice(choice_key, 0);
    let view2 = game.view().expect("view should succeed");
    let Object::Paragraph(line, _) = &view2.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(line.content(), "Styled static choice");
    assert!(line.spans[0].classes.is_empty(), "classes should be stripped by clean()");
    assert!(line.classes.is_empty());
}

#[ifengine::ifview]
fn page_choice_closure_cleaned_on_input(_: &mut TestState) {
    choice! {
        s!("Styled choice").cls("fade-in") => |l| {
            // Check that l received by closure is already cleaned
            assert!(l.spans[0].classes.is_empty(), "input to closure must be cleaned");
            l + " — processed"
        },
    };
}

#[test]
fn test_choice_closure_cleans_input_line() {
    let mut game = ifengine::Game::new_with_page(
        "page_choice_closure_cleaned_on_input",
        page_choice_closure_cleaned_on_input,
    );
    let view = game.view().expect("view should succeed");
    let choice_key = view.inner[0].id.expect("choice key");

    // Select choice 0
    game.inner.handle_choice(choice_key, 0);
    let view2 = game.view().expect("view should succeed");
    let Object::Paragraph(line, _) = &view2.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(line.content(), "Styled choice — processed");
}

#[ifengine::ifview]
fn page_replace_closure(state: &mut TestState) {
    replace!((100), "The iron chest is [[locked tight]]." => |l| {
        state.flag = true;
        l + " You broke the latch open!"
    });
    replace!((200), "The wooden door is [[ajar]]." => "The wooden door is wide open.");
}

#[test]
fn test_replace_closure_and_non_closure() {
    let mut game = ifengine::Game::new_with_page("page_replace_closure", page_replace_closure);
    let view = game.view().expect("view should succeed");

    // 1. Initial render for chest (key 100)
    let action_100 = if let Object::Paragraph(line, _) = &view.inner[0].object {
        assert_eq!(line.content(), "The iron chest is locked tight.");
        line.spans[1].action.clone().unwrap()
    } else {
        panic!("expected Object::Paragraph for chest");
    };

    // 2. Click chest link (key 100)
    game.handle_action(action_100).expect("action should succeed");
    let view2 = game.view().expect("view should succeed");
    if let Object::Paragraph(line, _) = &view2.inner[0].object {
        assert_eq!(
            line.content(),
            "The iron chest is locked tight. You broke the latch open!"
        );
        assert!(game.context.flag);
    } else {
        panic!("expected Object::Paragraph for chest");
    }

    // 3. Click door link (key 200) - non-closure
    let action_200 = if let Object::Paragraph(line, _) = &view2.inner[1].object {
        line.spans[1].action.clone().unwrap()
    } else {
        panic!("expected Object::Paragraph for door");
    };
    game.handle_action(action_200).expect("action should succeed");
    let view3 = game.view().expect("view should succeed");
    if let Object::Paragraph(line, _) = &view3.inner[1].object {
        assert_eq!(line.content(), "The wooden door is wide open.");
    } else {
        panic!("expected Object::Paragraph for door");
    }
}
