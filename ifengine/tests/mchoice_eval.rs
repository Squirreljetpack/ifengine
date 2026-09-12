use ifengine::elements::mchoice;
use ifengine::utils::MaskExt;
use ifengine::view::Object;

#[derive(Clone, Debug, Default)]
struct TestState {
    asked_key: bool,
    coins: u32,
}

#[ifengine::ifview]
fn page_mchoice_dialogue(state: &mut TestState) {
    let completed = mchoice! {
        "Ask about the key" => {
            state.asked_key = true;
            "The blacksmith nods. 'The key is hidden in the old well.'"
        },
        "Search the counter" => {
            state.coins += 5;
            "You find 5 shiny coins under a cloth."
        },
        "Silently inspect the forge" => {
            // Evaluates to (), so no paragraph pushed!
        },
    };

    if completed.all() {
        ifengine::elements::p!("All options exhausted.");
    }
}

#[test]
fn test_mchoice_eval_paragraphs_and_order() {
    let mut game = ifengine::Game::new_with_page("page_mchoice_dialogue", page_mchoice_dialogue);
    let view = game.view().expect("view should succeed");

    // Initially, 1 object: the Choice containing 3 options
    assert_eq!(view.inner.len(), 1);
    let choice_key = view.inner[0].id.expect("choice key");
    let Object::Choice(choices) = &view.inner[0].object else {
        panic!("expected Object::Choice");
    };
    assert_eq!(choices.len(), 3);

    // Choose option 0: "Ask about the key"
    game.inner.handle_choice(choice_key, 0);
    let view2 = game.view().expect("view should succeed");

    // Now we should have:
    // 1. Paragraph from option 0
    // 2. Choice with remaining 2 options (index 1 and 2)
    assert_eq!(view2.inner.len(), 2);
    let Object::Paragraph(p0, _) = &view2.inner[0].object else {
        panic!("expected Object::Paragraph for arm 0");
    };
    assert!(p0.content().contains("blacksmith"));
    assert!(game.context.asked_key);

    let Object::Choice(choices2) = &view2.inner[1].object else {
        panic!("expected Object::Choice for remaining choices pushed last");
    };
    assert_eq!(choices2.len(), 2);
    assert_eq!(choices2[0].0, 1);
    assert_eq!(choices2[1].0, 2);

    // Choose option 2: "Silently inspect the forge" (returns (), so no new paragraph)
    game.inner.handle_choice(choice_key, 2);
    let view3 = game.view().expect("view should succeed");

    // Still 2 objects: arm 0 paragraph + choice with remaining option 1
    assert_eq!(view3.inner.len(), 2);
    let Object::Choice(choices3) = &view3.inner[1].object else {
        panic!("expected Object::Choice");
    };
    assert_eq!(choices3.len(), 1);
    assert_eq!(choices3[0].0, 1);

    // Choose option 1: "Search the counter"
    game.inner.handle_choice(choice_key, 1);
    let view4 = game.view().expect("view should succeed");

    // All options selected!
    // 1. Paragraph from arm 0
    // 2. Paragraph from arm 1
    // (no Choice object pushed because choices are exhausted)
    // 3. Paragraph from completed.all()
    assert_eq!(view4.inner.len(), 3);
    let Object::Paragraph(p_arm1, _) = &view4.inner[1].object else {
        panic!("expected arm 1 paragraph");
    };
    assert!(p_arm1.content().contains("coins"));
    assert_eq!(game.context.coins, 5);

    let Object::Paragraph(p_all, _) = &view4.inner[2].object else {
        panic!("expected completed.all paragraph");
    };
    assert!(p_all.content().contains("exhausted"));
}
