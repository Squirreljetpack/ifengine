use ifengine::view::Object;

#[ifengine::ifview]
fn test_element_macro_page(_: &mut ()) {
    use ifengine::elements::{l, link, ps, s};
    let span = s!("span_x").cls("in").style("color", "blue");
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
        assert_eq!(s0.classes, vec!["in"]);
        assert_eq!(s0.style.get("color").map(|s| s.as_str()), Some("blue"));
    } else {
        panic!("expected Object::Paragraph");
    }
}

#[ifengine::ifview]
fn test_alts_page(_: &mut ()) {
    use ifengine::elements::{alts, ps};
    let a1 = alts!("option_a", "option_b");
    let a2 = alts!(["c1", "c2"], Cycle);
    ps!(a1, a2);
}

#[test]
fn test_alts_macro_and_keys() {
    let mut game = ifengine::Game::new_with_page("test_alts_page", test_alts_page);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 2);
    if let Object::Paragraph(l1) = &view.inner[0] {
        let s0 = &l1.spans[0];
        assert_eq!(s0.content, "option_a");
        let id0 = s0.id.expect("alts span must have an id");
        assert_eq!(id0 >> 48, 1);
        assert!(s0.action.is_some());
    } else {
        panic!("expected Object::Paragraph");
    }

    if let Object::Paragraph(l2) = &view.inner[1] {
        let s0 = &l2.spans[0];
        assert_eq!(s0.content, "c1");
        let id0 = s0.id.expect("alts span must have an id");
        assert_eq!(id0 >> 48, 1);
        assert!(s0.action.is_some());
    } else {
        panic!("expected Object::Paragraph");
    }
}

#[ifengine::ifview]
fn test_choice_and_element_ids_page(_: &mut ()) {
    use ifengine::elements::{choice, click, count, h, p, paragraph};

    h!("My Heading", 1);
    paragraph!("Intro text.");

    let c = count!(|val| format!("Clicked {val}"));
    let clk = click!("Action link", {});
    p!(c, clk);

    ifengine::elements::img!("https://example.com/test.png");

    choice! {
        "Pick A" => "Chose A",
        "Pick B" => "Chose B",
    };
}

#[test]
fn test_choice_and_element_ids() {
    let mut game = ifengine::Game::new_with_page("test_choice_and_element_ids_page", test_choice_and_element_ids_page);
    let view = game.view().expect("view should succeed");

    // 1. Heading does not have an ID (plain static text)
    if let Object::Heading(span, 1) = &view.inner[0] {
        assert!(span.id.is_none(), "heading span should not have an element id");
    } else {
        panic!("expected Object::Heading");
    }

    // 2. Paragraph does not have an ID on its Line (plain static text)
    if let Object::Paragraph(line) = &view.inner[1] {
        assert!(line.id.is_none(), "paragraph line should not have an element id");
    } else {
        panic!("expected Object::Paragraph");
    }

    // 3. Count and Click spans have IDs
    if let Object::Paragraph(line) = &view.inner[2] {
        assert!(line.spans[0].id.is_some(), "count span must have an element id");
        assert!(line.spans[1].id.is_some(), "click span must have an element id");
    } else {
        panic!("expected Object::Paragraph with count/click");
    }

    // 4. Image has an ID
    if let Object::Image(img) = &view.inner[3] {
        assert!(img.id.is_some(), "image must have an element id");
    } else {
        panic!("expected Object::Image");
    }

    // 5. Choice object has its key
    let choice_key = if let Object::Choice(key, choices) = &view.inner[4] {
        assert_eq!(choices.len(), 2);
        *key
    } else {
        panic!("expected Object::Choice");
    };

    // 6. Select Choice 0
    game.handle_choice(choice_key, 0);
    let updated_view = game.view().expect("view after choice should succeed");

    // 7. Choice transitioned to its replacement Paragraph with the EXACT SAME KEY!
    if let Object::Paragraph(line) = &updated_view.inner[4] {
        assert_eq!(line.id, Some(choice_key), "replacement paragraph must share the choice's key");
        assert_eq!(line.content(), "Chose A");
    } else {
        panic!("expected replacement Object::Paragraph");
    }
}

#[ifengine::ifview]
fn test_replace_page(_: &mut ()) {
    use ifengine::elements::replace;
    // Disappears on click
    replace!((100), "The chest is [[locked]].");
    // Replaces on click with simple expr
    replace!((200), "The door is [[closed]].", "The door is open.");
    // Replaces on click with arbitrary block expression
    replace!((300), "Click [[here]] for magic.", {
        let x = 2 + 2;
        format!("Magic number is {x}")
    });
}

#[test]
fn test_replace_macro() {
    let mut game = ifengine::Game::new_with_page("test_replace_page", test_replace_page);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 3);

    // Initial render for chest (key 100)
    let action_100 = if let Object::Paragraph(line) = &view.inner[0] {
        assert_eq!(line.id, Some(100));
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content, "The chest is ");
        assert_eq!(line.spans[1].content, "locked");
        assert!(line.spans[1].action.is_some());
        assert_eq!(line.spans[2].content, ".");
        line.spans[1].action.clone().unwrap()
    } else {
        panic!("expected Object::Paragraph for chest");
    };

    // Initial render for door (key 200)
    let action_200 = if let Object::Paragraph(line) = &view.inner[1] {
        assert_eq!(line.id, Some(200));
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content, "The door is ");
        assert_eq!(line.spans[1].content, "closed");
        assert!(line.spans[1].action.is_some());
        assert_eq!(line.spans[2].content, ".");
        line.spans[1].action.clone().unwrap()
    } else {
        panic!("expected Object::Paragraph for door");
    };

    // Initial render for magic (key 300)
    let action_300 = if let Object::Paragraph(line) = &view.inner[2] {
        assert_eq!(line.id, Some(300));
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content, "Click ");
        assert_eq!(line.spans[1].content, "here");
        assert!(line.spans[1].action.is_some());
        assert_eq!(line.spans[2].content, " for magic.");
        line.spans[1].action.clone().unwrap()
    } else {
        panic!("expected Object::Paragraph for magic");
    };

    // Click chest link (key 100)
    game.handle_action(action_100).expect("action should succeed");
    let view2 = game.view().expect("view should succeed");

    // Chest is now completely disappeared from the view (length reduced from 3 to 2)
    assert_eq!(view2.inner.len(), 2);
    if let Object::Paragraph(line) = &view2.inner[0] {
        assert_eq!(line.id, Some(200));
    } else {
        panic!("expected door at index 0 after chest disappears");
    }

    // Click door link (key 200)
    game.handle_action(action_200).expect("action should succeed");
    let view3 = game.view().expect("view should succeed");

    // Door is now replaced with "The door is open." with the exact same key 200
    assert_eq!(view3.inner.len(), 2);
    if let Object::Paragraph(line) = &view3.inner[0] {
        assert_eq!(line.id, Some(200));
        assert_eq!(line.content(), "The door is open.");
    } else {
        panic!("expected Object::Paragraph for door");
    }

    // Click magic link (key 300)
    game.handle_action(action_300).expect("action should succeed");
    let view4 = game.view().expect("view should succeed");

    // Magic is now replaced with the block's evaluated result
    if let Object::Paragraph(line) = &view4.inner[1] {
        assert_eq!(line.id, Some(300));
        assert_eq!(line.content(), "Magic number is 4");
    } else {
        panic!("expected Object::Paragraph for magic");
    }
}

#[ifengine::ifview]
fn test_interpolation_page(_: &mut ()) {
    use ifengine::elements::{h, l, link, p, ps, s, text, texts};

    let name = "Sen";
    let score = 42;
    let gold = 100;

    h!("Welcome to {name}'s Quest", 1);
    p!("Hello {name}, you have {gold} gold.");
    ps!("Score: {score}", "Player: {name}");
    text!("Stats: {gold} gold");
    texts!("Gold: {gold}", "Score: {score}");

    let line = l!("Line with {name} and score {score}");
    p!(line);

    let span = s!("Badge: {name}");
    let lnk = link!("Visit {name}");
    p!(span, lnk);

    let uncopied = String::from("Unmoved");
    p!("First: {uncopied}");
    p!("Second: {uncopied}");
}

#[test]
fn test_variable_interpolation_macros() {
    let mut game = ifengine::Game::new_with_page("test_interpolation_page", test_interpolation_page);
    let view = game.view().expect("view should succeed");

    // 0: h!
    if let Object::Heading(span, 1) = &view.inner[0] {
        assert_eq!(span.content, "Welcome to Sen’s Quest");
    } else {
        panic!("expected Heading for index 0");
    }

    // 1: p! with interpolation
    if let Object::Paragraph(line) = &view.inner[1] {
        assert_eq!(line.content(), "Hello Sen, you have 100 gold.");
        assert_eq!(line.spans.len(), 5); // "Hello ", "Sen", ", you have ", "100", " gold."
    } else {
        panic!("expected Paragraph for index 1");
    }

    // 2 & 3: ps! with interpolation
    if let Object::Paragraph(line) = &view.inner[2] {
        assert_eq!(line.content(), "Score: 42");
    } else {
        panic!("expected Paragraph for index 2");
    }
    if let Object::Paragraph(line) = &view.inner[3] {
        assert_eq!(line.content(), "Player: Sen");
    } else {
        panic!("expected Paragraph for index 3");
    }

    // 4: text! with interpolation
    if let Object::Text(line, _) = &view.inner[4] {
        assert_eq!(line.content(), "Stats: 100 gold");
    } else {
        panic!("expected Text for index 4");
    }

    // 5 & 6: texts! with interpolation
    if let Object::Text(line, _) = &view.inner[5] {
        assert_eq!(line.content(), "Gold: 100");
    } else {
        panic!("expected Text for index 5");
    }
    if let Object::Text(line, _) = &view.inner[6] {
        assert_eq!(line.content(), "Score: 42");
    } else {
        panic!("expected Text for index 6");
    }

    // 7: p!(l!("Line with {name} and score {score}"))
    if let Object::Paragraph(line) = &view.inner[7] {
        assert_eq!(line.content(), "Line with Sen and score 42");
    } else {
        panic!("expected Paragraph for index 7");
    }

    // 8: p!(s!("Badge: {name}"), link!("Visit {name}"))
    if let Object::Paragraph(line) = &view.inner[8] {
        assert_eq!(line.spans[0].content, "Badge: Sen");
        assert_eq!(line.spans[1].content, "Visit Sen");
        assert!(matches!(line.spans[1].variant, ifengine::view::SpanVariant::Link));
    } else {
        panic!("expected Paragraph for index 8");
    }

    // 9 & 10: non-copy string borrowing
    if let Object::Paragraph(line) = &view.inner[9] {
        assert_eq!(line.content(), "First: Unmoved");
    } else {
        panic!("expected Paragraph for index 9");
    }
    if let Object::Paragraph(line) = &view.inner[10] {
        assert_eq!(line.content(), "Second: Unmoved");
    } else {
        panic!("expected Paragraph for index 10");
    }
}
