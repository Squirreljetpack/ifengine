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
    let mut game =
        ifengine::Game::new_with_page("test_element_macro_page", test_element_macro_page);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 1);
    assert!(
        view.inner[0].id.is_some(),
        "paragraph must have an object id"
    );
    if let Object::Paragraph(line, _) = &view.inner[0].object {
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
    let a1 = alts!(["option_a", "option_b"]);
    let a2 = alts!(["c1", "c2"], Cycle);
    ps!(a1, a2);
}

#[test]
fn test_alts_macro_and_keys() {
    let mut game = ifengine::Game::new_with_page("test_alts_page", test_alts_page);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 2);
    if let Object::Paragraph(l1, _) = &view.inner[0].object {
        let s0 = &l1.spans[0];
        assert_eq!(s0.content, "option_a");
        let id0 = s0.id.expect("alts span must have an id");
        assert_eq!(id0 >> 48, 1);
        assert!(s0.action.is_some());
    } else {
        panic!("expected Object::Paragraph");
    }

    if let Object::Paragraph(l2, _) = &view.inner[1].object {
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
    let mut game = ifengine::Game::new_with_page(
        "test_choice_and_element_ids_page",
        test_choice_and_element_ids_page,
    );
    let view = game.view().expect("view should succeed");

    // 1. Heading has an ID on its StampedObject, inner span has no ID
    assert!(view.inner[0].id.is_some(), "heading must have an object id");
    if let Object::Heading(span, 1) = &view.inner[0].object {
        assert!(
            span.id.is_none(),
            "heading span should not have an element id"
        );
    } else {
        panic!("expected Object::Heading");
    }

    // 2. Paragraph has an ID on its StampedObject, inner line has no ID
    assert!(
        view.inner[1].id.is_some(),
        "paragraph must have an object id"
    );
    if let Object::Paragraph(line, _) = &view.inner[1].object {
        assert_eq!(line.spans.len(), 1);
    } else {
        panic!("expected Object::Paragraph");
    }

    // 3. Count and Click spans have IDs, and outer paragraph has an ID
    assert!(
        view.inner[2].id.is_some(),
        "paragraph with count/click must have an object id"
    );
    if let Object::Paragraph(line, _) = &view.inner[2].object {
        assert!(
            line.spans[0].id.is_some(),
            "count span must have an element id"
        );
        assert!(
            line.spans[1].id.is_some(),
            "click span must have an element id"
        );
    } else {
        panic!("expected Object::Paragraph with count/click");
    }

    // 4. Image has an ID on StampedObject
    assert!(view.inner[3].id.is_some(), "image must have an object id");
    if !matches!(&view.inner[3].object, Object::Image(_)) {
        panic!("expected Object::Image");
    }

    // 5. Choice object has its key
    let choice_key = if let Object::Choice(choices) = &view.inner[4].object {
        assert_eq!(choices.len(), 2);
        view.inner[4].id.expect("choice must have key")
    } else {
        panic!("expected Object::Choice");
    };

    // 6. Select Choice 0
    game.handle_choice(choice_key, 0);
    let updated_view = game.view().expect("view after choice should succeed");

    // 7. Choice transitioned to its replacement Paragraph with the EXACT SAME KEY!
    assert_eq!(
        updated_view.inner[4].id,
        Some(choice_key),
        "replacement paragraph must share the choice's key"
    );
    if let Object::Paragraph(line, _) = &updated_view.inner[4].object {
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
    // Replaces when any link bit is clicked
    replace!((400), "Choose [[left]] or [[right]].", "You moved forward.");
}

#[test]
fn test_replace_macro() {
    let mut game = ifengine::Game::new_with_page("test_replace_page", test_replace_page);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 4);

    // Initial render for chest (key 100)
    assert_eq!(view.inner[0].id, Some(100));
    let action_100 = if let Object::Paragraph(line, _) = &view.inner[0].object {
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
    assert_eq!(view.inner[1].id, Some(200));
    let action_200 = if let Object::Paragraph(line, _) = &view.inner[1].object {
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
    assert_eq!(view.inner[2].id, Some(300));
    let action_300 = if let Object::Paragraph(line, _) = &view.inner[2].object {
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content, "Click ");
        assert_eq!(line.spans[1].content, "here");
        assert!(line.spans[1].action.is_some());
        assert_eq!(line.spans[2].content, " for magic.");
        line.spans[1].action.clone().unwrap()
    } else {
        panic!("expected Object::Paragraph for magic");
    };

    // Initial render for multi-link (key 400)
    assert_eq!(view.inner[3].id, Some(400));
    let action_400_right = if let Object::Paragraph(line, _) = &view.inner[3].object {
        assert_eq!(line.spans.len(), 5);
        assert_eq!(line.spans[0].content, "Choose ");
        assert_eq!(line.spans[1].content, "left");
        assert_eq!(line.spans[2].content, " or ");
        assert_eq!(line.spans[3].content, "right");
        assert!(line.spans[3].action.is_some());
        assert_eq!(line.spans[4].content, ".");
        line.spans[3].action.clone().unwrap()
    } else {
        panic!("expected Object::Paragraph for multi-link");
    };

    // Click chest link (key 100)
    game.handle_action(action_100)
        .expect("action should succeed");
    let view2 = game.view().expect("view should succeed");

    // Chest is now completely disappeared from the view (length reduced from 4 to 3)
    assert_eq!(view2.inner.len(), 3);
    assert_eq!(view2.inner[0].id, Some(200));
    assert!(matches!(&view2.inner[0].object, Object::Paragraph(..)));

    // Click door link (key 200)
    game.handle_action(action_200)
        .expect("action should succeed");
    let view3 = game.view().expect("view should succeed");

    // Door is now replaced with "The door is open." with the exact same key 200
    assert_eq!(view3.inner.len(), 3);
    assert_eq!(view3.inner[0].id, Some(200));
    if let Object::Paragraph(line, _) = &view3.inner[0].object {
        assert_eq!(line.content(), "The door is open.");
    } else {
        panic!("expected Object::Paragraph for door");
    }

    // Click magic link (key 300)
    game.handle_action(action_300)
        .expect("action should succeed");
    let view4 = game.view().expect("view should succeed");

    // Magic is now replaced with the block's evaluated result
    assert_eq!(view4.inner[1].id, Some(300));
    if let Object::Paragraph(line, _) = &view4.inner[1].object {
        assert_eq!(line.content(), "Magic number is 4");
    } else {
        panic!("expected Object::Paragraph for magic");
    }

    // Click the second link in multi-link (bit 1 of key 400)
    game.handle_action(action_400_right)
        .expect("action should succeed");
    let view5 = game.view().expect("view should succeed");

    assert_eq!(view5.inner[2].id, Some(400));
    if let Object::Paragraph(line, _) = &view5.inner[2].object {
        assert_eq!(line.content(), "You moved forward.");
    } else {
        panic!("expected Object::Paragraph for key 400");
    }
}

#[ifengine::ifview]
fn test_interpolation_page(_: &mut ()) {
    use ifengine::elements::{dparagraph, h, l, link, mparagraph, p, ps, s};

    let name = "Sen";
    let score = 42;
    let gold = 100;

    h!("Welcome to {name}'s Quest", 1);
    p!("Hello {name}, you have {gold} gold.");
    ps!("Score: {score}", "Player: {name}");
    p!("Stats: {gold} gold");
    ps!("Gold: {gold}", "Score: {score}");

    let line = l!("Line with {name} and score {score}");
    p!(line);

    let span = s!("Badge: {name}");
    let lnk = link!("Visit {name}");
    p!(span, lnk);

    let uncopied = String::from("Unmoved");
    p!("First: {uncopied}");
    p!("Second: {uncopied}");

    mparagraph!((500), "Take the [[{name}'s blade]].");
    dparagraph!((600), "Travel to [[{name}'s camp]].");
}

#[test]
fn test_variable_interpolation_macros() {
    let mut game =
        ifengine::Game::new_with_page("test_interpolation_page", test_interpolation_page);
    let view = game.view().expect("view should succeed");

    // 0: h!
    if let Object::Heading(span, 1) = &view.inner[0].object {
        assert!(span.content.contains("Sen"));
    } else {
        panic!("expected Heading for index 0");
    }

    // 1: p! with interpolation
    if let Object::Paragraph(line, _) = &view.inner[1].object {
        assert!(line.content().contains("Sen"));
        assert!(line.content().contains("100"));
        assert_eq!(line.spans.len(), 5); // "Hello ", "Sen", ", you have ", "100", " gold."
    } else {
        panic!("expected Paragraph for index 1");
    }

    // 2 & 3: ps! with interpolation
    if let Object::Paragraph(line, _) = &view.inner[2].object {
        assert!(line.content().contains("42"));
    } else {
        panic!("expected Paragraph for index 2");
    }
    if let Object::Paragraph(line, _) = &view.inner[3].object {
        assert!(line.content().contains("Sen"));
    } else {
        panic!("expected Paragraph for index 3");
    }

    // 4: p! with interpolation
    if let Object::Paragraph(line, _) = &view.inner[4].object {
        assert!(line.content().contains("100"));
    } else {
        panic!("expected Paragraph for index 4");
    }

    // 5 & 6: ps! with interpolation
    if let Object::Paragraph(line, _) = &view.inner[5].object {
        assert!(line.content().contains("100"));
    } else {
        panic!("expected Paragraph for index 5");
    }
    if let Object::Paragraph(line, _) = &view.inner[6].object {
        assert!(line.content().contains("42"));
    } else {
        panic!("expected Paragraph for index 6");
    }

    // 7: p!(l!("Line with {name} and score {score}"))
    if let Object::Paragraph(line, _) = &view.inner[7].object {
        assert!(line.content().contains("Sen"));
        assert!(line.content().contains("42"));
    } else {
        panic!("expected Paragraph for index 7");
    }

    // 8: p!(s!("Badge: {name}"), link!("Visit {name}"))
    if let Object::Paragraph(line, _) = &view.inner[8].object {
        assert!(line.spans[0].content.contains("Sen"));
        assert!(line.spans[1].content.contains("Sen"));
        assert!(matches!(
            line.spans[1].variant,
            ifengine::view::SpanVariant::Link
        ));
    } else {
        panic!("expected Paragraph for index 8");
    }

    // 9 & 10: non-copy string borrowing
    if let Object::Paragraph(line, _) = &view.inner[9].object {
        assert!(line.content().contains("Unmoved"));
    } else {
        panic!("expected Paragraph for index 9");
    }
    if let Object::Paragraph(line, _) = &view.inner[10].object {
        assert!(line.content().contains("Unmoved"));
    } else {
        panic!("expected Paragraph for index 10");
    }

    // 11: mparagraph! with interpolation
    if let Object::Paragraph(line, _) = &view.inner[11].object {
        assert!(line.content().contains("Sen"));
    } else {
        panic!("expected Paragraph for index 11");
    }

    // 12: dparagraph! with interpolation
    if let Object::Paragraph(line, _) = &view.inner[12].object {
        assert!(line.content().contains("Sen"));
    } else {
        panic!("expected Paragraph for index 12");
    }
}

#[derive(Debug, Default, Clone)]
struct ClickState {
    unbounded_count: usize,
    bounded_count: usize,
}

#[ifengine::ifview]
fn test_click_page(state: &mut ClickState) {
    use ifengine::elements::{click, p};

    let clk_unbounded = click!("Unbounded", {
        state.unbounded_count += 1;
    });

    let clk_bounded = click!(
        "Bounded",
        {
            state.bounded_count += 1;
        },
        2
    );

    p!(clk_unbounded, clk_bounded);
}

#[test]
fn test_click_macro_repeatable_and_max_clicks() {
    use ifengine::view::Object;

    let mut game = ifengine::Game::new_with_page("test_click_page", test_click_page);

    let view1 = game.view().expect("view 1 should succeed");
    assert_eq!(game.context.unbounded_count, 0);
    assert_eq!(game.context.bounded_count, 0);

    let Object::Paragraph(line, _) = &view1.inner[0].object else {
        panic!("expected paragraph");
    };
    let unbounded_action = line.spans[0].action.clone().expect("unbounded action");
    let bounded_action = line.spans[1].action.clone().expect("bounded action");

    // Click unbounded 1st time
    game.handle_action(unbounded_action.clone()).unwrap();
    let _ = game.view().unwrap();
    assert_eq!(game.context.unbounded_count, 1);

    // Re-render without clicking: code should NOT run again (dirty bit was cleared)
    let _ = game.view().unwrap();
    assert_eq!(game.context.unbounded_count, 1);

    // Click unbounded 2nd time
    game.handle_action(unbounded_action.clone()).unwrap();
    let _ = game.view().unwrap();
    assert_eq!(game.context.unbounded_count, 2);

    // Click unbounded 3rd time
    game.handle_action(unbounded_action).unwrap();
    let _ = game.view().unwrap();
    assert_eq!(game.context.unbounded_count, 3);

    // Click bounded 1st time (max 2)
    game.handle_action(bounded_action.clone()).unwrap();
    let _ = game.view().unwrap();
    assert_eq!(game.context.bounded_count, 1);

    // Click bounded 2nd time (max 2)
    game.handle_action(bounded_action.clone()).unwrap();
    let _ = game.view().unwrap();
    assert_eq!(game.context.bounded_count, 2);

    // Click bounded 3rd time: exceeds max_clicks (2), should NOT run
    game.handle_action(bounded_action).unwrap();
    let _ = game.view().unwrap();
    assert_eq!(
        game.context.bounded_count, 2,
        "bounded click should not execute beyond max_clicks"
    );
}

#[derive(Debug, Default, Clone)]
struct DparagraphState {
    last_target: String,
}

#[ifengine::ifview]
fn test_dparagraph_page(state: &mut DparagraphState) {
    use ifengine::elements::dparagraph;

    match dparagraph!("Go to the [[forest]] or [[]]the [[inn]].") {
        "forest" => state.last_target = "forest".to_string(),
        "inn" => state.last_target = "inn".to_string(),
        "" => {}
        _ => state.last_target = "unknown".to_string(),
    }
}

#[test]
fn test_dparagraph_interaction() {
    let mut game = ifengine::Game::new_with_page("test_dparagraph_page", test_dparagraph_page);
    let view = game.view().expect("view should succeed");

    assert_eq!(game.context.last_target, "");

    let Object::Paragraph(line, _) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };

    let forest_span = line
        .spans
        .iter()
        .find(|s| s.content.trim() == "forest")
        .unwrap();
    let inn_span = line
        .spans
        .iter()
        .find(|s| s.content.trim() == "inn")
        .unwrap();

    let forest_action = forest_span.action.as_ref().unwrap().clone();
    let inn_action = inn_span.action.as_ref().unwrap().clone();

    // Click forest
    game.handle_action(forest_action).unwrap();
    let _ = game.view().unwrap();
    assert_eq!(game.context.last_target, "forest");

    // Click inn
    game.handle_action(inn_action).unwrap();
    let _ = game.view().unwrap();
    assert_eq!(game.context.last_target, "inn");
}

#[derive(Debug, Default, Clone)]
struct DparagraphSequenceState {
    last_target: String,
    clicked_custom_link: bool,
}

#[ifengine::ifview]
fn test_dparagraph_seq_dest_page(state: &mut DparagraphSequenceState) {
    state.clicked_custom_link = true;
}

#[ifengine::ifview]
fn test_dparagraph_seq_page(state: &mut DparagraphSequenceState) {
    use ifengine::elements::{dparagraph, link, s};

    match dparagraph!(
        s!("You see "),
        "a [[sword]]",
        link!(" (examine)", test_dparagraph_seq_dest_page),
        " and a [[shield]]."
    ) {
        "sword" => state.last_target = "sword".to_string(),
        "shield" => state.last_target = "shield".to_string(),
        _ => {}
    }
}

#[test]
fn test_dparagraph_sequence_of_spans() {
    let mut game =
        ifengine::Game::new_with_page("test_dparagraph_seq_page", test_dparagraph_seq_page);
    let view = game.view().expect("view should succeed");

    assert_eq!(game.context.last_target, "");
    assert!(!game.context.clicked_custom_link);

    let Object::Paragraph(line, _) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };

    let sword_span = line.spans.iter().find(|s| s.content == "sword").unwrap();
    let examine_span = line
        .spans
        .iter()
        .find(|s| s.content.contains("examine"))
        .unwrap();
    let shield_span = line.spans.iter().find(|s| s.content == "shield").unwrap();

    let sword_action = sword_span.action.as_ref().unwrap().clone();
    let examine_action = examine_span.action.as_ref().unwrap().clone();
    let shield_action = shield_span.action.as_ref().unwrap().clone();

    // Click custom link
    game.handle_action(examine_action).unwrap();
    let _ = game.view().unwrap();
    assert!(game.context.clicked_custom_link);

    // Click dynamic link: sword
    let mut game2 =
        ifengine::Game::new_with_page("test_dparagraph_seq_page", test_dparagraph_seq_page);
    let _ = game2.view().unwrap();
    game2.handle_action(sword_action).unwrap();
    let _ = game2.view().unwrap();
    assert_eq!(game2.context.last_target, "sword");

    // Click dynamic link: shield
    let mut game3 =
        ifengine::Game::new_with_page("test_dparagraph_seq_page", test_dparagraph_seq_page);
    let _ = game3.view().unwrap();
    game3.handle_action(shield_action).unwrap();
    let _ = game3.view().unwrap();
    assert_eq!(game3.context.last_target, "shield");
}

#[derive(Debug, Default, Clone)]
struct MparagraphSequenceState {
    mask: Vec<bool>,
}

#[ifengine::ifview]
fn test_mparagraph_seq_page(state: &mut MparagraphSequenceState) {
    use ifengine::elements::{link, mparagraph, s};

    state.mask = mparagraph!(
        s!("Equipment: "),
        "take [[torch]]",
        link!(" (help)", test_dparagraph_seq_dest_page),
        " and [[map]]" :: "eq-trailer"
    );
}

#[test]
fn test_mparagraph_sequence_of_spans() {
    let mut game =
        ifengine::Game::new_with_page("test_mparagraph_seq_page", test_mparagraph_seq_page);
    let view = game.view().expect("view should succeed");

    // 2 dynamic links: torch and map
    assert_eq!(game.context.mask, vec![false, false]);

    let Object::Paragraph(line, trailer) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };
    assert_eq!(*trailer, "eq-trailer");

    let torch_span = line.spans.iter().find(|s| s.content == "torch").unwrap();
    let map_span = line.spans.iter().find(|s| s.content == "map").unwrap();

    let torch_action = torch_span.action.as_ref().unwrap().clone();
    let map_action = map_span.action.as_ref().unwrap().clone();

    // Click torch
    game.handle_action(torch_action).unwrap();
    let view2 = game.view().unwrap();
    assert_eq!(game.context.mask, vec![true, false]);

    let Object::Paragraph(line2, _) = &view2.inner[0].object else {
        panic!("expected Paragraph");
    };
    let torch_span2 = line2.spans.iter().find(|s| s.content == "torch").unwrap();
    let map_span2 = line2.spans.iter().find(|s| s.content == "map").unwrap();
    assert!(
        torch_span2.action.is_none(),
        "already clicked torch should have no action"
    );
    assert!(
        map_span2.action.is_some(),
        "unclicked map should still have an action"
    );

    // Click map
    game.handle_action(map_action).unwrap();
    let view3 = game.view().unwrap();
    assert_eq!(game.context.mask, vec![true, true]);

    let Object::Paragraph(line3, _) = &view3.inner[0].object else {
        panic!("expected Paragraph");
    };
    let torch_span3 = line3.spans.iter().find(|s| s.content == "torch").unwrap();
    let map_span3 = line3.spans.iter().find(|s| s.content == "map").unwrap();
    assert!(
        torch_span3.action.is_none(),
        "already clicked torch should have no action"
    );
    assert!(
        map_span3.action.is_none(),
        "already clicked map should have no action"
    );
}

#[ifengine::ifview]
fn test_get_set_page(_: &mut ()) {
    use ifengine::elements::{get, p, set};

    // 1-arg get initially None
    assert_eq!(get!("door_open"), None);

    // 1-arg set sets to Some(0)
    set!("door_open");
    assert_eq!(get!("door_open"), Some(0));

    // 2-arg set sets to specified value
    set!("gold_count", 50);
    assert_eq!(get!("gold_count"), Some(50));

    // 2-arg get returns Span when Some, Span::default() when None
    let span_present = get!("door_open", "Door is open!");
    assert_eq!(span_present.content, "Door is open!");

    let span_absent = get!("missing_flag", "Should not show");
    assert_eq!(span_absent.content, "");

    // 3-arg get evaluates when_some or when_none
    let span_some = get!("gold_count", "Has gold", "No gold");
    assert_eq!(span_some.content, "Has gold");

    let span_none = get!("missing_item", "Has item", "No item");
    assert_eq!(span_none.content, "No item");

    // String interpolation in get!
    let player_level = 5;
    let span_interp = get!("gold_count", "Level: {player_level}");
    assert_eq!(span_interp.content, "Level: 5");

    // Non-string key
    set!(1234u64, 99);
    assert_eq!(get!(1234u64), Some(99));

    // Embedded in paragraph
    p!(get!("door_open", "Entryway: Open", "Entryway: Closed"));
}

#[test]
fn test_get_and_set_macros() {
    let mut game = ifengine::Game::new_with_page("test_get_set_page", test_get_set_page);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 1);
    if let Object::Paragraph(line, _) = &view.inner[0].object {
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "Entryway: Open");
    } else {
        panic!("expected Paragraph");
    }
}

#[ifengine::ifview]
fn test_replace_span_page(_: &mut ()) {
    use ifengine::elements::{p, replace_line, replace_span, reps};

    // Test replace_span inside a paragraph
    let sp = replace_span!("locked" => "unlocked");
    let sp2 = reps!("closed" => "opened");
    let sp3 = reps!("sealed" => {
        let prefix = "broken";
        format!("{prefix} lock")
    });
    p!(
        "The chest is ",
        sp,
        " and the door is ",
        sp2,
        " and vault is ",
        sp3,
        "."
    );

    // Test replace_line
    let _ = replace_line!("The gate is [[shut]]" => "The gate is open");
}

#[test]
fn test_replace_span_and_line() {
    let mut game = ifengine::Game::new_with_page("test_replace_span_page", test_replace_span_page);
    let view = game.view().expect("view should succeed");

    let Object::Paragraph(line, _) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };

    let locked_span = line.spans.iter().find(|s| s.content == "locked").unwrap();
    let sealed_span = line.spans.iter().find(|s| s.content == "sealed").unwrap();
    assert!(
        locked_span.action.is_some(),
        "unclicked locked span should have action"
    );
    assert!(
        sealed_span.action.is_some(),
        "unclicked sealed span should have action"
    );

    let locked_action = locked_span.action.as_ref().unwrap().clone();
    let sealed_action = sealed_span.action.as_ref().unwrap().clone();
    game.handle_action(locked_action).unwrap();

    let view2 = game.view().unwrap();
    let Object::Paragraph(line2, _) = &view2.inner[0].object else {
        panic!("expected Paragraph");
    };
    let unlocked_span = line2
        .spans
        .iter()
        .find(|s| s.content == "unlocked")
        .unwrap();
    assert!(
        unlocked_span.action.is_none(),
        "clicked span should be replaced and have no action"
    );

    game.handle_action(sealed_action).unwrap();
    let view3 = game.view().unwrap();
    let Object::Paragraph(line3, _) = &view3.inner[0].object else {
        panic!("expected Paragraph");
    };
    let broken_span = line3
        .spans
        .iter()
        .find(|s| s.content == "broken lock")
        .unwrap();
    assert!(broken_span.action.is_none());
}

#[derive(Debug, Default, Clone)]
struct DparagraphHashedState {
    chest_opened: bool,
    last_clicked: String,
}

#[ifengine::ifview]
fn test_dparagraph_hashed_key_page(state: &mut DparagraphHashedState) {
    use ifengine::elements::{dparagraph, get};

    state.last_clicked = dparagraph!("You see a [[chest]] and a [[window]].").to_string();

    if get!("chest").is_some() {
        state.chest_opened = true;
    }
}

#[test]
fn test_dparagraph_hashed_key_state() {
    let mut game = ifengine::Game::new_with_page(
        "test_dparagraph_hashed_key_page",
        test_dparagraph_hashed_key_page,
    );
    let view = game.view().expect("view should succeed");

    assert_eq!(game.context.last_clicked, "");
    assert!(!game.context.chest_opened);

    let Object::Paragraph(line, _) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };

    let chest_span = line.spans.iter().find(|s| s.content == "chest").unwrap();
    let chest_action = chest_span.action.as_ref().unwrap().clone();

    // Click chest
    game.handle_action(chest_action.clone()).unwrap();
    let _ = game.view().unwrap();

    // Return value is "chest"
    assert_eq!(game.context.last_clicked, "chest");
    // get!("chest").is_some() is true!
    assert!(game.context.chest_opened);

    // On subsequent render without clicks, last_clicked becomes "" but chest_opened remains true!
    let _ = game.view().unwrap();
    assert_eq!(game.context.last_clicked, "");
    assert!(game.context.chest_opened);

    // Clicking chest again re-increments counter and triggers again on next render
    game.handle_action(chest_action).unwrap();
    let _ = game.view().unwrap();
    assert_eq!(game.context.last_clicked, "chest");
}

#[derive(Debug, Default, Clone)]
struct ActionRuleState {
    dp_clicked: String,
    mp_mask: Vec<bool>,
}

#[ifengine::ifview]
fn test_action_rule_page(state: &mut ActionRuleState) {
    use ifengine::elements::{dparagraph, mparagraph};
    use ifengine::view::Span;

    let dynamic_prefix = format!("Dynamic [[{}]] or ", "cave");
    let unbraced_action_span =
        Span::from("[[preserved_target]]").with_action(ifengine::Action::SetBit(999, 0));

    state.dp_clicked = dparagraph!(dynamic_prefix, unbraced_action_span, " [[ruins]]").to_string();

    state.mp_mask = mparagraph!(
        "Choose [[alpha]], ",
        Span::from("[[beta_link]]").with_action(ifengine::Action::SetBit(888, 0)),
        " or [[gamma]]"
    );
}

#[test]
fn test_action_rule_behavior() {
    let mut game = ifengine::Game::new_with_page("test_action_rule_page", test_action_rule_page);
    let view = game.view().unwrap();

    // Verify dparagraph structure
    let Object::Paragraph(dp_line, _) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };

    // The dynamic "cave" was split
    assert!(
        dp_line
            .spans
            .iter()
            .any(|s| s.content == "cave"
                && matches!(s.action, Some(ifengine::Action::SetDirty(_, _))))
    );
    // The span with existing action was PRESERVED as-is (not split)
    let preserved = dp_line
        .spans
        .iter()
        .find(|s| s.content == "[[preserved_target]]")
        .unwrap();
    assert!(matches!(
        preserved.action,
        Some(ifengine::Action::SetBit(999, 0))
    ));
    // The "ruins" link was split
    assert!(dp_line.spans.iter().any(
        |s| s.content == "ruins" && matches!(s.action, Some(ifengine::Action::SetDirty(_, _)))
    ));

    // Verify mparagraph structure
    let Object::Paragraph(mp_line, _) = &view.inner[1].object else {
        panic!("expected Paragraph");
    };

    // alpha has action SetBit(key, 0)
    let alpha = mp_line.spans.iter().find(|s| s.content == "alpha").unwrap();
    assert!(matches!(alpha.action, Some(ifengine::Action::SetBit(_, 0))));

    // beta_link preserved existing action
    let beta = mp_line
        .spans
        .iter()
        .find(|s| s.content == "[[beta_link]]")
        .unwrap();
    assert!(matches!(
        beta.action,
        Some(ifengine::Action::SetBit(888, 0))
    ));

    // gamma has action SetBit(key, 1) because beta was not split
    let gamma = mp_line.spans.iter().find(|s| s.content == "gamma").unwrap();
    assert!(matches!(gamma.action, Some(ifengine::Action::SetBit(_, 1))));

    // Click alpha in mparagraph
    let alpha_action = alpha.action.clone().unwrap();
    game.handle_action(alpha_action).unwrap();
    let view2 = game.view().unwrap();

    let Object::Paragraph(mp_line2, _) = &view2.inner[1].object else {
        panic!("expected Paragraph");
    };
    // Once clicked, alpha has action = None!
    let alpha2 = mp_line2
        .spans
        .iter()
        .find(|s| s.content == "alpha")
        .unwrap();
    assert!(alpha2.action.is_none());

    // Click dynamic "cave" in dparagraph
    let cave_action = dp_line
        .spans
        .iter()
        .find(|s| s.content == "cave")
        .unwrap()
        .action
        .clone()
        .unwrap();
    game.handle_action(cave_action).unwrap();
    let _ = game.view().unwrap();
    // dparagraph polled dynamic target "cave"!
    assert_eq!(game.context.dp_clicked, "cave");
}

#[derive(Debug, Default, Clone)]
struct DparagraphConditionState {
    clicked: String,
    disable_chest: bool,
    reset_chest: bool,
    zero_chest: bool,
}

#[ifengine::ifview]
fn test_dparagraph_condition_page(state: &mut DparagraphConditionState) {
    use ifengine::elements::{dparagraph, reset_key, set};

    if state.disable_chest {
        set!("chest", 12345);
    }
    if state.reset_chest {
        reset_key!("chest");
    }
    if state.zero_chest {
        set!("chest", 0);
    }

    state.clicked = dparagraph!("Examine [[chest]] or [[door]].").to_string();
}

#[test]
fn test_dparagraph_braced_span_attachment_rules() {
    let mut game = ifengine::Game::new_with_page(
        "test_dparagraph_condition_page",
        test_dparagraph_condition_page,
    );

    // Initial render: hash keys are None -> both "chest" and "door" have actions attached
    let view = game.view().unwrap();
    let Object::Paragraph(line, _) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };
    let chest_span = line.spans.iter().find(|s| s.content == "chest").unwrap();
    let door_span = line.spans.iter().find(|s| s.content == "door").unwrap();
    assert!(chest_span.action.is_some());
    assert!(door_span.action.is_some());

    // Click door -> polls "door", and stores loc (non-zero) in state
    let door_action = door_span.action.as_ref().unwrap().clone();
    game.handle_action(door_action).unwrap();
    let view2 = game.view().unwrap();
    assert_eq!(game.context.clicked, "door");
    let Object::Paragraph(line2, _) = &view2.inner[0].object else {
        panic!("expected Paragraph");
    };
    let door_span2 = line2.spans.iter().find(|s| s.content == "door").unwrap();
    assert!(door_span2.action.is_none());

    // Next render: door's state has value = loc (non-zero and not None) -> action is NOT attached
    let view2b = game.view().unwrap();
    let Object::Paragraph(line2b, _) = &view2b.inner[0].object else {
        panic!("expected Paragraph");
    };
    let door_span2b = line2b.spans.iter().find(|s| s.content == "door").unwrap();
    assert!(door_span2b.action.is_none());

    // Disable chest by setting it to a custom value 12345
    game.context.disable_chest = true;
    let view3 = game.view().unwrap();
    let Object::Paragraph(line3, _) = &view3.inner[0].object else {
        panic!("expected Paragraph");
    };
    let chest_span3 = line3.spans.iter().find(|s| s.content == "chest").unwrap();
    assert!(chest_span3.action.is_none());

    // Reset chest key -> hash key is None again -> action IS attached
    game.context.disable_chest = false;
    game.context.reset_chest = true;
    let view4 = game.view().unwrap();
    let Object::Paragraph(line4, _) = &view4.inner[0].object else {
        panic!("expected Paragraph");
    };
    let chest_span4 = line4.spans.iter().find(|s| s.content == "chest").unwrap();
    assert!(chest_span4.action.is_some());

    // Zero chest key -> hash key is Some(0) -> action IS attached
    game.context.reset_chest = false;
    game.context.zero_chest = true;
    let view5 = game.view().unwrap();
    let Object::Paragraph(line5, _) = &view5.inner[0].object else {
        panic!("expected Paragraph");
    };
    let chest_span5 = line5.spans.iter().find(|s| s.content == "chest").unwrap();
    assert!(chest_span5.action.is_some());
}

#[derive(Debug, Default, Clone)]
struct YieldTestState {
    step: usize,
    unlock: bool,
}

#[ifengine::ifview]
fn test_yield_page(state: &mut YieldTestState) {
    use ifengine::elements::{YIELD, p, set};

    if state.unlock {
        set!("unlock", 1);
    }

    p!("Before yield");
    state.step += 1;

    // Conditionally yield based on whether "unlock" key is set:
    // When "unlock" is None -> yields immediately.
    // When "unlock" is Some -> does not yield, continues execution.
    YIELD!("unlock");

    state.step += 10;
    p!("After yield");
}

#[test]
fn test_yield_macro_behavior() {
    let mut game = ifengine::Game::new_with_page("test_yield_page", test_yield_page);
    let view = game.view().unwrap();
    assert_eq!(view.inner.len(), 1);
    assert_eq!(game.context.step, 1);

    game.context.unlock = true;
    let view2 = game.view().unwrap();
    assert_eq!(view2.inner.len(), 2);
    assert_eq!(game.context.step, 12);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CustomKey {
    Score,
    Flags,
}

impl ifengine::core::key::IntoPageKey for CustomKey {
    fn into_page_key(self) -> ifengine::PageKey {
        match self {
            CustomKey::Score => ifengine::core::key::hash_key("custom_score"),
            CustomKey::Flags => ifengine::core::key::hash_key("custom_flags"),
        }
    }
}

#[ifengine::ifview]
fn test_trait_based_keys_page(_: &mut ()) {
    use ifengine::elements::{
        get, inc_key, p, read_key_mask, reset_key, set, set_key_mask, unset_key_mask,
    };

    // 1. &str variable
    let str_key: &str = "str_var_key";
    set!(str_key, 42);
    let val_str = get!(str_key);
    assert_eq!(val_str, Some(42));
    assert_eq!(get!("str_var_key"), Some(42)); // Matches literal hash!

    // 2. String / &String
    let string_key: String = format!("item_{}", 7);
    set!(string_key.clone(), 100);
    assert_eq!(get!(&string_key), Some(100));
    assert_eq!(get!("item_7"), Some(100));

    // 3. Integer variable (usize, u32, i32, u64)
    let int_u32: u32 = 1001;
    let int_usize: usize = 2002;
    let int_i32: i32 = 3003;
    let int_u64: u64 = 4004;

    set!(int_u32, 10);
    set!(int_usize, 20);
    set!(int_i32, 30);
    set!(int_u64, 40);

    assert_eq!(get!(int_u32), Some(10));
    assert_eq!(get!(int_usize), Some(20));
    assert_eq!(get!(int_i32), Some(30));
    assert_eq!(get!(int_u64), Some(40));

    // 4. inc_key! on str and int
    inc_key!(str_key);
    assert_eq!(get!(str_key), Some(43));
    inc_key!(int_u32);
    assert_eq!(get!(int_u32), Some(11));

    // 5. Mask operations on dynamic keys
    let mask_key = "mask_key";
    reset_key!(mask_key);
    set_key_mask!(mask_key, 0, 2);
    let mask: [bool; 4] = read_key_mask!(mask_key, 4);
    assert!(mask[0]);
    assert!(!mask[1]);
    assert!(mask[2]);
    assert!(!mask[3]);

    unset_key_mask!(mask_key, 0);
    let mask_after: [bool; 4] = read_key_mask!(mask_key, 4);
    assert!(!mask_after[0]);
    assert!(mask_after[2]);

    // 6. reset_key! on str and int
    reset_key!(str_key);
    assert_eq!(get!(str_key), None);
    assert_eq!(get!("str_var_key"), None);

    reset_key!(int_u32);
    assert_eq!(get!(int_u32), None);

    // 7. Custom type implementing IntoPageKey
    set!(CustomKey::Score, 999);
    set!(CustomKey::Flags, 1);
    assert_eq!(get!(CustomKey::Score), Some(999));
    assert_eq!(get!("custom_score"), Some(999));
    assert_eq!(get!(CustomKey::Flags), Some(1));
    assert_eq!(get!("custom_flags"), Some(1));

    // 8. MaybeKey with str literal, str variable, and int
    use ifengine::elements::{alts, count, mchoice};
    let cnt_str_key = "cnt_key";
    let _c1 = count!(("literal_cnt"), |n| format!("Clicked {n} times"));
    let _c2 = count!((cnt_str_key), |n| format!("Clicked {n} times"));
    let _c3 = count!((42), |n| format!("Clicked {n} times"));

    let alt_key = "alts_key";
    let _a1 = alts!(("literal_alt"), ["a", "b"]);
    let _a2 = alts!((alt_key), ["a", "b"]);
    let _a3 = alts!((99), ["a", "b"]);

    let choice_key = "choice_key";
    mchoice!(
        (choice_key),
        "Option 1" => {},
        "Option 2" => {},
    );

    p!("Trait-based keys tested successfully");
}

#[test]
fn test_trait_based_keys() {
    let mut game =
        ifengine::Game::new_with_page("test_trait_based_keys_page", test_trait_based_keys_page);
    let view = game.view().expect("view should succeed");
    assert_eq!(view.inner.len(), 2);

    let choice_obj = &view.inner[0];
    assert_eq!(
        choice_obj.id,
        Some(ifengine::core::key::hash_key("choice_key"))
    );
}
