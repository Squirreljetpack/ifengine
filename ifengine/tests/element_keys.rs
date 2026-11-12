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

        assert_eq!(line.spans.len(), 4);
        let s0 = &line.spans[1];
        let s1 = &line.spans[2];

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
    let a2 = alts!(Cycle: ["c1", "c2"]);
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
        assert!(s0.active());
    } else {
        panic!("expected Object::Paragraph");
    }

    if let Object::Paragraph(l2, _) = &view.inner[1].object {
        let s0 = &l2.spans[0];
        assert_eq!(s0.content, "c1");
        let id0 = s0.id.expect("alts span must have an id");
        assert_eq!(id0 >> 48, 1);
        assert!(s0.active());
    } else {
        panic!("expected Object::Paragraph");
    }
}

#[ifengine::ifview]
fn test_choice_and_element_ids_page(_: &mut ()) {
    use ifengine::elements::{choice, click, h, p, paragraph};

    h!("My Heading", 1);
    paragraph!("Intro text.");

    let c = click!(|val| format!("Clicked {val}"));
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

    // 3. Click spans have IDs, and outer paragraph has an ID
    assert!(
        view.inner[2].id.is_some(),
        "paragraph with click spans must have an object id"
    );
    if let Object::Paragraph(line, _) = &view.inner[2].object {
        assert!(
            line.spans[0].id.is_some(),
            "first click span must have an element id"
        );
        assert!(
            line.spans[1].id.is_some(),
            "second click span must have an element id"
        );
    } else {
        panic!("expected Object::Paragraph with click spans");
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
    replace!((100), "The chest is [locked].");
    // Replaces on click with simple expr
    replace!((200), "The door is [closed].", "The door is open.");
    // Replaces on click with arbitrary block expression
    replace!((300), "Click [here] for magic.", {
        let x = 2 + 2;
        format!("Magic number is {x}")
    });
    // Replaces when any link bit is clicked
    replace!((400), "Choose [left] or [right].", "You moved forward.");
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
        assert!(line.spans[1].active());
        assert_eq!(line.spans[2].content, ".");
        line.spans[1].action.clone()
    } else {
        panic!("expected Object::Paragraph for chest");
    };

    // Initial render for door (key 200)
    assert_eq!(view.inner[1].id, Some(200));
    let action_200 = if let Object::Paragraph(line, _) = &view.inner[1].object {
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content, "The door is ");
        assert_eq!(line.spans[1].content, "closed");
        assert!(line.spans[1].active());
        assert_eq!(line.spans[2].content, ".");
        line.spans[1].action.clone()
    } else {
        panic!("expected Object::Paragraph for door");
    };

    // Initial render for magic (key 300)
    assert_eq!(view.inner[2].id, Some(300));
    let action_300 = if let Object::Paragraph(line, _) = &view.inner[2].object {
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content, "Click ");
        assert_eq!(line.spans[1].content, "here");
        assert!(line.spans[1].active());
        assert_eq!(line.spans[2].content, " for magic.");
        line.spans[1].action.clone()
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
        assert!(line.spans[3].active());
        assert_eq!(line.spans[4].content, ".");
        line.spans[3].action.clone()
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
    use ifengine::elements::{h, l, link, p, paragraph, ps, s};

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

    paragraph!((500), "Take the [{name}'s blade].");
    paragraph!((600), "Travel to [{name}'s camp].");
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
        assert_eq!(line.spans.len(), 6);
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

    // 11: paragraph! with interpolation (key 500)
    if let Object::Paragraph(line, _) = &view.inner[11].object {
        assert!(line.content().contains("Sen"));
    } else {
        panic!("expected Paragraph for index 11");
    }

    // 12: paragraph! with interpolation (key 600)
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
    disabled_count: usize,
}

#[ifengine::ifview]
fn test_click_page(state: &mut ClickState) {
    use ifengine::elements::{click, p};

    let clk_unbounded = click!("Unbounded", {
        state.unbounded_count += 1;
    });

    let clk_bounded = click!(
        2: "Bounded",
        {
            state.bounded_count += 1;
        }
    );

    let clk_disabled = click!(0: "Disabled", {
        state.disabled_count += 1;
    });

    p!(clk_unbounded, clk_bounded, clk_disabled);
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
    let unbounded_action = line.spans[0].action.clone();
    let bounded_action = line.spans[1].action.clone();
    assert!(
        !line.spans[2].active(),
        "max_clicks 0 should attach no action"
    );

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
    let view_after_2 = game.view().unwrap();
    assert_eq!(game.context.bounded_count, 2);
    let Object::Paragraph(line, _) = &view_after_2.inner[0].object else {
        panic!("expected paragraph");
    };
    assert!(
        !line.spans[1].active(),
        "when count >= max_clicks, no action should be attached"
    );

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
    use ifengine::elements::paragraph;

    match paragraph!("Go to the [forest] or []the [inn].").as_str() {
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

    let forest_action = forest_span.action.clone();
    let inn_action = inn_span.action.clone();

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
struct DparagraphMaskState {
    clicked: String,
    bits: u64,
    key: u64,
    mask: Vec<bool>,
}

#[ifengine::ifview]
fn test_dparagraph_mask_page(state: &mut DparagraphMaskState) {
    use ifengine::elements::paragraph;

    let data = paragraph!("Go to the [forest] or the [inn].");
    state.clicked = data.as_str().to_string();
    state.bits = data.bits;
    state.key = data.key();
    state.mask = data.mask(2);
}

#[test]
fn test_dparagraph_mask_tracks_each_link() {
    let mut game =
        ifengine::Game::new_with_page("test_dparagraph_mask_page", test_dparagraph_mask_page);
    let view = game.view().expect("view should succeed");

    assert_eq!(game.context.mask, vec![false, false]);
    assert_ne!(game.context.key, 0);

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
    let forest_action = forest_span.action.clone();
    let inn_action = inn_span.action.clone();

    // Clicking the second link sets only bit 1 on the paragraph key.
    game.handle_action(inn_action).unwrap();
    let _ = game.view().unwrap();
    assert_eq!(game.context.clicked, "inn");
    assert_eq!(game.context.bits, 0b10);
    assert_eq!(game.context.mask, vec![false, true]);

    // Clicking the first link sets its bit too, without clearing the other.
    game.handle_action(forest_action).unwrap();
    let _ = game.view().unwrap();
    assert_eq!(game.context.clicked, "forest");
    assert_eq!(game.context.bits, 0b11);
    assert_eq!(game.context.mask, vec![true, true]);
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
    use ifengine::elements::{link, paragraph, s};

    match paragraph!(
        s!("You see "),
        "a [sword]",
        link!(" (examine)", test_dparagraph_seq_dest_page),
        " and a [shield]."
    )
    .as_str()
    {
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

    let sword_action = sword_span.action.clone();
    let examine_action = examine_span.action.clone();
    let shield_action = shield_span.action.clone();

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
    last_clicked: String,
}

#[ifengine::ifview]
fn test_mparagraph_seq_page(state: &mut MparagraphSequenceState) {
    use ifengine::elements::{link, paragraph, s};

    state.last_clicked = paragraph!(
        s!("Equipment: "),
        "take [torch]",
        link!(" (help)", test_dparagraph_seq_dest_page),
        " and [map]" :: "eq-trailer"
    )
    .unwrap_or_default();
}

#[test]
fn test_mparagraph_sequence_of_spans() {
    let mut game =
        ifengine::Game::new_with_page("test_mparagraph_seq_page", test_mparagraph_seq_page);
    let view = game.view().expect("view should succeed");

    assert_eq!(game.context.last_clicked, "");

    let Object::Paragraph(line, trailer) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };
    assert_eq!(*trailer, "eq-trailer");

    let torch_span = line.spans.iter().find(|s| s.content == "torch").unwrap();
    let map_span = line.spans.iter().find(|s| s.content == "map").unwrap();

    let torch_action = torch_span.action.clone();
    let map_action = map_span.action.clone();

    // Click torch: torch becomes inactive; map stays active
    game.handle_action(torch_action).unwrap();
    let view2 = game.view().unwrap();
    assert_eq!(game.context.last_clicked, "torch");

    let Object::Paragraph(line2, _) = &view2.inner[0].object else {
        panic!("expected Paragraph");
    };
    let torch_span2 = line2.spans.iter().find(|s| s.content == "torch").unwrap();
    let map_span2 = line2.spans.iter().find(|s| s.content == "map").unwrap();
    assert!(
        !torch_span2.active(),
        "already clicked torch should have no action"
    );
    assert!(
        map_span2.active(),
        "unclicked map should still have an action"
    );

    // Click map: map becomes inactive
    game.handle_action(map_action).unwrap();
    let view3 = game.view().unwrap();
    assert_eq!(game.context.last_clicked, "map");

    let Object::Paragraph(line3, _) = &view3.inner[0].object else {
        panic!("expected Paragraph");
    };
    let torch_span3 = line3.spans.iter().find(|s| s.content == "torch").unwrap();
    let map_span3 = line3.spans.iter().find(|s| s.content == "map").unwrap();
    assert!(
        !torch_span3.active(),
        "already clicked torch should have no action"
    );
    assert!(
        !map_span3.active(),
        "already clicked map should have no action"
    );
}

#[ifengine::ifview]
fn test_get_set_page(_: &mut ()) {
    use ifengine::elements::{get, l, p, set};

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
    assert_eq!(span_present.spans[0].content, "Door is open!");

    let span_absent = get!("missing_flag", "Should not show");
    assert!(span_absent.spans.is_empty());

    // Arrow syntax: lhs is result when not present, rhs when present
    let span_arrow = get!("door_open", "Door is closed" => "Door is open");
    assert_eq!(span_arrow.spans[0].content, "Door is open");

    let span_arrow_none = get!("missing_door", "Door is closed" => "Door is open");
    assert_eq!(span_arrow_none.spans[0].content, "Door is closed");

    // Reversed arrow: the leading side is the present case
    let span_rev = get!("door_open", "Door is open" <= "Door is closed");
    assert_eq!(span_rev.spans[0].content, "Door is open");

    let span_rev_none = get!("missing_door", "Door is open" <= "Door is closed");
    assert_eq!(span_rev_none.spans[0].content, "Door is closed");

    // Multiple spans come from a single `Into<Line>` expression:
    let span_multi = get!("door_open", l!("Part A", ", ", "Part B"));
    assert_eq!(span_multi.content(), "Part A, Part B");

    // String interpolation in get!
    let player_level = 5;
    let span_interp = get!("gold_count", "Level: {player_level}");
    assert_eq!(span_interp.spans[0].content, "Level: 5");

    // Non-string key
    set!(1234u64, 99);
    assert_eq!(get!(1234u64), Some(99));

    // Embedded in paragraph
    p!(get!("door_open", "Entryway: Closed" => "Entryway: Open"));
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
    use ifengine::elements::{l, p, s};

    // Test s! replacement inside a paragraph
    let sp = s!("locked" => "unlocked");
    let sp2 = s!("closed" => "opened");
    let sp3 = s!("sealed" => {
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

    // Test l! replacement
    let _ = l!("The gate is [shut]" => "The gate is open");
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
        locked_span.active(),
        "unclicked locked span should have action"
    );
    assert!(
        sealed_span.active(),
        "unclicked sealed span should have action"
    );

    let locked_action = locked_span.action.clone();
    let sealed_action = sealed_span.action.clone();
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
        !unlocked_span.active(),
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
    assert!(!broken_span.active());
}

#[derive(Debug, Default, Clone)]
struct DparagraphHashedState {
    chest_opened: bool,
    last_clicked: String,
}

#[ifengine::ifview]
fn test_dparagraph_hashed_key_page(state: &mut DparagraphHashedState) {
    use ifengine::elements::{get, paragraph};

    state.last_clicked = paragraph!("You see a [chest] and a [window].").unwrap_or_default();

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
    let chest_action = chest_span.action.clone();

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
    p_clicked: String,
}

#[ifengine::ifview]
fn test_action_rule_page(state: &mut ActionRuleState) {
    use ifengine::elements::paragraph;
    use ifengine::view::Span;

    let dynamic_prefix = format!("Dynamic [{}] or ", "cave");
    let unbraced_action_span =
        Span::from("[preserved_target]").with_action(ifengine::Action::SetBit(999, 0));

    state.dp_clicked =
        paragraph!(dynamic_prefix, unbraced_action_span, " [ruins]").unwrap_or_default();

    state.p_clicked = paragraph!(
        "Choose [alpha], ",
        Span::from("[beta_link]").with_action(ifengine::Action::SetBit(888, 0)),
        " or [gamma]"
    )
    .unwrap_or_default();
}

#[test]
fn test_action_rule_behavior() {
    let mut game = ifengine::Game::new_with_page("test_action_rule_page", test_action_rule_page);
    let view = game.view().unwrap();

    // Verify first paragraph structure
    let Object::Paragraph(dp_line, _) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };

    // The dynamic "cave" was split and gets SetDirty action
    assert!(dp_line.spans.iter().any(|s| s.content == "cave"
        && matches!(s.action.first(), Some(ifengine::Action::SetDirty(_, _)))));
    // The span with existing action was PRESERVED as-is (not split)
    let preserved = dp_line
        .spans
        .iter()
        .find(|s| s.content == "[preserved_target]")
        .unwrap();
    assert!(matches!(
        preserved.action.first(),
        Some(ifengine::Action::SetBit(999, 0))
    ));
    // The "ruins" link was split and gets SetDirty action
    assert!(dp_line.spans.iter().any(|s| s.content == "ruins"
        && matches!(s.action.first(), Some(ifengine::Action::SetDirty(_, _)))));

    // Verify second paragraph: [alpha] and [gamma] get SetDirty; [beta_link] preserves existing action
    let Object::Paragraph(p_line, _) = &view.inner[1].object else {
        panic!("expected Paragraph");
    };

    // alpha gets SetDirty (new paragraph! behavior)
    let alpha = p_line.spans.iter().find(|s| s.content == "alpha").unwrap();
    assert!(matches!(
        alpha.action.first(),
        Some(ifengine::Action::SetDirty(_, _))
    ));

    // beta_link preserved existing action (not split because it already has an action)
    let beta = p_line
        .spans
        .iter()
        .find(|s| s.content == "[beta_link]")
        .unwrap();
    assert!(matches!(
        beta.action.first(),
        Some(ifengine::Action::SetBit(888, 0))
    ));

    // gamma gets SetDirty
    let gamma = p_line.spans.iter().find(|s| s.content == "gamma").unwrap();
    assert!(matches!(
        gamma.action.first(),
        Some(ifengine::Action::SetDirty(_, _))
    ));

    // Click alpha: it should become inactive on next render
    let alpha_action = alpha.action.clone();
    game.handle_action(alpha_action).unwrap();
    let view2 = game.view().unwrap();

    let Object::Paragraph(p_line2, _) = &view2.inner[1].object else {
        panic!("expected Paragraph");
    };
    let alpha2 = p_line2.spans.iter().find(|s| s.content == "alpha").unwrap();
    assert!(!alpha2.active(), "clicked alpha should be inactive");

    // Click dynamic "cave" in first paragraph
    let cave_action = dp_line
        .spans
        .iter()
        .find(|s| s.content == "cave")
        .unwrap()
        .action
        .clone();
    game.handle_action(cave_action).unwrap();
    let _ = game.view().unwrap();
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
    use ifengine::elements::{paragraph, reset_key, set};

    if state.disable_chest {
        set!("chest", 12345);
    }
    if state.reset_chest {
        reset_key!("chest");
    }
    if state.zero_chest {
        set!("chest", 0);
    }

    state.clicked = paragraph!("Examine [chest] or [door].").unwrap_or_default();
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
    assert!(chest_span.active());
    assert!(door_span.active());

    // Click door -> polls "door", and stores loc (non-zero) in state
    let door_action = door_span.action.clone();
    game.handle_action(door_action).unwrap();
    let view2 = game.view().unwrap();
    assert_eq!(game.context.clicked, "door");
    let Object::Paragraph(line2, _) = &view2.inner[0].object else {
        panic!("expected Paragraph");
    };
    let door_span2 = line2.spans.iter().find(|s| s.content == "door").unwrap();
    assert!(!door_span2.active());

    // Next render: door's state has value = loc (non-zero and not None) -> action is NOT attached
    let view2b = game.view().unwrap();
    let Object::Paragraph(line2b, _) = &view2b.inner[0].object else {
        panic!("expected Paragraph");
    };
    let door_span2b = line2b.spans.iter().find(|s| s.content == "door").unwrap();
    assert!(!door_span2b.active());

    // Disable chest by setting it to a custom value 12345
    game.context.disable_chest = true;
    let view3 = game.view().unwrap();
    let Object::Paragraph(line3, _) = &view3.inner[0].object else {
        panic!("expected Paragraph");
    };
    let chest_span3 = line3.spans.iter().find(|s| s.content == "chest").unwrap();
    assert!(!chest_span3.active());

    // Reset chest key -> hash key is None again -> action IS attached
    game.context.disable_chest = false;
    game.context.reset_chest = true;
    let view4 = game.view().unwrap();
    let Object::Paragraph(line4, _) = &view4.inner[0].object else {
        panic!("expected Paragraph");
    };
    let chest_span4 = line4.spans.iter().find(|s| s.content == "chest").unwrap();
    assert!(chest_span4.active());

    // Zero chest key -> hash key is Some(0) -> action IS attached
    game.context.reset_chest = false;
    game.context.zero_chest = true;
    let view5 = game.view().unwrap();
    let Object::Paragraph(line5, _) = &view5.inner[0].object else {
        panic!("expected Paragraph");
    };
    let chest_span5 = line5.spans.iter().find(|s| s.content == "chest").unwrap();
    assert!(chest_span5.active());
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

#[derive(Debug, Default, Clone)]
struct YieldMultiTestState {
    step: usize,
    unlock_a: bool,
    unlock_b: bool,
}

#[ifengine::ifview]
fn test_yield_multi_page(state: &mut YieldMultiTestState) {
    use ifengine::elements::{YIELD, p, set};

    if state.unlock_a {
        set!("unlock_a", 1);
    }
    if state.unlock_b {
        set!("unlock_b", 1);
    }

    p!("Before yield");
    state.step += 1;

    // Yields only if none of the keys are is_some
    YIELD!("unlock_a" | "unlock_b");

    state.step += 10;
    p!("After yield");
}

#[test]
fn test_yield_multi_keys() {
    // 1. Neither key set -> both None -> yields immediately
    let mut game = ifengine::Game::new_with_page("test_yield_multi_page", test_yield_multi_page);
    let view = game.view().unwrap();
    assert_eq!(view.inner.len(), 1);
    assert_eq!(game.context.step, 1);

    // 2. Only unlock_a set -> unlock_a is Some -> does not yield
    let mut game_a = ifengine::Game::new_with_page("test_yield_multi_page", test_yield_multi_page);
    game_a.context.unlock_a = true;
    let view_a = game_a.view().unwrap();
    assert_eq!(view_a.inner.len(), 2);
    assert_eq!(game_a.context.step, 11);

    // 3. Only unlock_b set -> unlock_b is Some -> does not yield
    let mut game_b = ifengine::Game::new_with_page("test_yield_multi_page", test_yield_multi_page);
    game_b.context.unlock_b = true;
    let view_b = game_b.view().unwrap();
    assert_eq!(view_b.inner.len(), 2);
    assert_eq!(game_b.context.step, 11);

    // 4. Both keys set -> both are Some -> does not yield
    let mut game_both =
        ifengine::Game::new_with_page("test_yield_multi_page", test_yield_multi_page);
    game_both.context.unlock_a = true;
    game_both.context.unlock_b = true;
    let view_both = game_both.view().unwrap();
    assert_eq!(view_both.inner.len(), 2);
    assert_eq!(game_both.context.step, 11);
}

#[derive(Debug, Default, Clone)]
struct ClickHashState {
    seen: bool,
}

#[ifengine::ifview]
fn test_click_content_hash_page(state: &mut ClickHashState) {
    use ifengine::elements::{click, get, p};

    p!(
        "on to a ",
        click!("rocky"),
        " path high on the side of a valley."
    );

    if get!("rocky").is_some() {
        state.seen = true;
    }
}

#[test]
fn test_click_readable_by_content_hash() {
    let mut game =
        ifengine::Game::new_with_page("test_click_content_hash_page", test_click_content_hash_page);
    let view = game.view().unwrap();
    assert!(!game.context.seen);

    let Object::Paragraph(line, _) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };
    let rocky = line.spans.iter().find(|s| s.content == "rocky").unwrap();
    let action = rocky.action.clone();

    game.handle_action(action).unwrap();
    let _ = game.view().unwrap();
    assert!(
        game.context.seen,
        "get!(\"rocky\") must observe a body-less click!"
    );
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
    use ifengine::elements::{alts, click, l, mchoice, s};
    let cnt_str_key = "cnt_key";
    let _c1 = click!(("literal_cnt"), |n| format!("Clicked {n} times"));
    let _c2 = click!((cnt_str_key), |n| format!("Clicked {n} times"));
    let _c3 = click!((42), |n| format!("Clicked {n} times"));

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

    let span_with_key = s!(("stream"), "stream");
    assert_eq!(span_with_key.content, "stream");
    assert_eq!(
        span_with_key.id,
        Some(ifengine::core::key::IntoPageKey::into_page_key("stream"))
    );

    let line_with_key = l!(("line_key"), "line");
    assert_eq!(line_with_key.content(), "line");

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

#[ifengine::ifview]
fn test_line_concatenation_page(_s: &mut ()) {
    use ifengine::elements::{extend, l, link, p, s};
    let sub = l!(
        s!("Prefix: ").cls("bold"),
        link!("Go next", test_line_concatenation_page)
    );
    p!(sub, " - postfix text.");

    p!("Start: ");
    extend!(l!(
        "Extended ",
        link!("click target", test_line_concatenation_page)
    ));
}

#[test]
fn test_nested_line_concatenation() {
    let mut game =
        ifengine::Game::new_with_page("test_line_concatenation_page", test_line_concatenation_page);
    let view = game.view().expect("view should succeed");
    assert_eq!(view.inner.len(), 2);

    // Check first paragraph
    if let Object::Paragraph(line, _) = &view.inner[0].object {
        assert_eq!(line.spans.len(), 5);
        assert_eq!(line.spans[1].classes, vec!["bold"]);
        assert!(matches!(
            line.spans[2].variant,
            ifengine::view::SpanVariant::Link
        ));
        assert!(line.spans[2].active());
        assert_eq!(line.spans[4].content, " - postfix text.");
    } else {
        panic!("expected Paragraph for index 0");
    }

    // Check extended paragraph
    if let Object::Paragraph(line, _) = &view.inner[1].object {
        assert_eq!(line.spans.len(), 5);
        assert_eq!(line.spans[0].content, "Start: ");
        assert_eq!(line.spans[2].content, "Extended ");
        assert!(matches!(
            line.spans[3].variant,
            ifengine::view::SpanVariant::Link
        ));
        assert!(line.spans[3].active());
    } else {
        panic!("expected Paragraph for index 1");
    }
}

#[derive(Debug, Default, Clone)]
struct DparagraphLimitState {
    last_clicked: String,
}

#[ifengine::ifview]
fn test_dparagraph_limit_1_page(state: &mut DparagraphLimitState) {
    use ifengine::elements::paragraph;
    state.last_clicked = paragraph!(1: "Choose [apple] or [banana]").unwrap_or_default();
}

#[test]
fn test_dparagraph_max_clicks_1() {
    let mut game =
        ifengine::Game::new_with_page("test_dparagraph_limit_1_page", test_dparagraph_limit_1_page);

    // Initial render: both apple and banana have actions
    let view = game.view().unwrap();
    let Object::Paragraph(line, _) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };
    let apple_span = line.spans.iter().find(|s| s.content == "apple").unwrap();
    let banana_span = line.spans.iter().find(|s| s.content == "banana").unwrap();
    assert!(apple_span.active());
    assert!(banana_span.active());
    assert_eq!(game.context.last_clicked, "");

    let apple_action = apple_span.action.clone();

    // Click apple -> apple polled, and since 1 >= 1, ALL links are disabled!
    game.handle_action(apple_action).unwrap();
    let view2 = game.view().unwrap();
    assert_eq!(game.context.last_clicked, "apple");

    let Object::Paragraph(line2, _) = &view2.inner[0].object else {
        panic!("expected Paragraph");
    };
    let apple_span2 = line2.spans.iter().find(|s| s.content == "apple").unwrap();
    let banana_span2 = line2.spans.iter().find(|s| s.content == "banana").unwrap();
    assert!(!apple_span2.active(), "apple should be disabled");
    assert!(
        !banana_span2.active(),
        "banana should also be disabled after limit reached"
    );

    // Subsequent render: links remain disabled, last_clicked is ""
    let view3 = game.view().unwrap();
    assert_eq!(game.context.last_clicked, "");
    let Object::Paragraph(line3, _) = &view3.inner[0].object else {
        panic!("expected Paragraph");
    };
    assert!(line3.spans.iter().all(|s| !s.active()));
}

#[ifengine::ifview]
fn test_dparagraph_limit_0_page(state: &mut DparagraphLimitState) {
    use ifengine::elements::paragraph;
    state.last_clicked = paragraph!(0: "Choose [apple] or [banana]").unwrap_or_default();
}

#[test]
fn test_dparagraph_max_clicks_0_disables_all_links() {
    let mut game =
        ifengine::Game::new_with_page("test_dparagraph_limit_0_page", test_dparagraph_limit_0_page);

    let view = game.view().unwrap();
    let Object::Paragraph(line, _) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };
    let apple_span = line.spans.iter().find(|s| s.content == "apple").unwrap();
    let banana_span = line.spans.iter().find(|s| s.content == "banana").unwrap();
    assert!(!apple_span.active(), "apple should be disabled");
    assert!(!banana_span.active(), "banana should be disabled");
    assert_eq!(game.context.last_clicked, "");
}

#[ifengine::ifview]
fn test_dparagraph_limit_2_page(state: &mut DparagraphLimitState) {
    use ifengine::elements::paragraph;
    state.last_clicked = paragraph!(2: "Take [sword], [shield], or [potion]" :: ["highlight", "extra-class"], "loot-trailer").unwrap_or_default();
}

#[test]
fn test_dparagraph_max_clicks_2_with_trailer() {
    let mut game =
        ifengine::Game::new_with_page("test_dparagraph_limit_2_page", test_dparagraph_limit_2_page);

    let view = game.view().unwrap();
    let Object::Paragraph(line, trailer) = &view.inner[0].object else {
        panic!("expected Paragraph");
    };
    assert_eq!(*trailer, "loot-trailer");
    assert_eq!(
        line.classes,
        vec!["highlight".to_string(), "extra-class".to_string()]
    );

    let sword_action = line
        .spans
        .iter()
        .find(|s| s.content == "sword")
        .unwrap()
        .action
        .clone();
    let shield_span = line.spans.iter().find(|s| s.content == "shield").unwrap();
    let potion_span = line.spans.iter().find(|s| s.content == "potion").unwrap();
    assert!(shield_span.active());
    assert!(potion_span.active());

    // Click sword (1st click): 1 < 2, so sword is disabled, but shield and potion remain active
    game.handle_action(sword_action).unwrap();
    let view2 = game.view().unwrap();
    assert_eq!(game.context.last_clicked, "sword");

    let Object::Paragraph(line2, _) = &view2.inner[0].object else {
        panic!("expected Paragraph");
    };
    let sword_span2 = line2.spans.iter().find(|s| s.content == "sword").unwrap();
    let shield_span2 = line2.spans.iter().find(|s| s.content == "shield").unwrap();
    let potion_span2 = line2.spans.iter().find(|s| s.content == "potion").unwrap();
    assert!(!sword_span2.active());
    assert!(shield_span2.active());
    assert!(potion_span2.active());

    let shield_action2 = shield_span2.action.clone();

    // Click shield (2nd click): 2 >= 2, so ALL links are now disabled
    game.handle_action(shield_action2).unwrap();
    let view3 = game.view().unwrap();
    assert_eq!(game.context.last_clicked, "shield");

    let Object::Paragraph(line3, _) = &view3.inner[0].object else {
        panic!("expected Paragraph");
    };
    assert!(
        line3.spans.iter().all(|s| !s.active()),
        "all links disabled when >= 2 clicked"
    );
}
