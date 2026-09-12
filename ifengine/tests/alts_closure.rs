use ifengine::elements::{alts, ps};
use ifengine::view::Object;

#[derive(Clone, Debug, Default)]
struct TestState {
    stop_idx: usize,
    cycle_idx: usize,
    bracket_idx: usize,
}

#[ifengine::ifview]
fn page_alts_test(state: &mut TestState) {
    let s1 = alts!(["alpha", "beta", "gamma"], |idx| {
        state.stop_idx = idx;
    });

    let s2 = alts!(["red", "green", "blue"], Cycle, |idx| {
        state.cycle_idx = idx;
    });

    let s3 = alts!(["one", "two"], |idx| {
        state.bracket_idx = idx;
    });

    ps!(s1, s2, s3);
}

#[test]
fn test_alts_closure_initial_and_clicks() {
    let mut game = ifengine::Game::new_with_page("page_alts_test", page_alts_test);
    let view = game.view().expect("view should succeed");

    assert_eq!(view.inner.len(), 3);

    // Verify initial values passed to closures
    assert_eq!(game.context.stop_idx, 0);
    assert_eq!(game.context.cycle_idx, 0);
    assert_eq!(game.context.bracket_idx, 0);

    // Paragraph 0: Stop variant ("alpha", "beta", "gamma")
    let Object::Paragraph(p0, _) = &view.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(p0.spans[0].content, "alpha");

    // Paragraph 1: Cycle variant ("red", "green", "blue")
    let Object::Paragraph(p1, _) = &view.inner[1].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(p1.spans[0].content, "red");

    // Paragraph 2: Bracketed Stop variant ("one", "two")
    let Object::Paragraph(p2, _) = &view.inner[2].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(p2.spans[0].content, "one");

    // Click s1 (Stop) -> should advance to index 1 ("beta")
    let action_stop = p0.spans[0].action.clone().expect("action");
    game.inner.handle_action(action_stop).expect("action");
    let view2 = game.view().expect("view should succeed");
    assert_eq!(game.context.stop_idx, 1);
    let Object::Paragraph(p0_2, _) = &view2.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(p0_2.spans[0].content, "beta");

    // Click s1 (Stop) again -> should advance to index 2 ("gamma")
    let action_stop_2 = p0_2.spans[0].action.clone().expect("action");
    game.inner.handle_action(action_stop_2).expect("action");
    let view3 = game.view().expect("view should succeed");
    assert_eq!(game.context.stop_idx, 2);
    let Object::Paragraph(p0_3, _) = &view3.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(p0_3.spans[0].content, "gamma");
    assert!(p0_3.spans[0].action.is_none());

    // Click s2 (Cycle) -> should advance to index 1 ("green")
    let action_cycle = p1.spans[0].action.clone().expect("action");
    game.inner.handle_action(action_cycle).expect("action");
    let view5 = game.view().expect("view should succeed");
    assert_eq!(game.context.cycle_idx, 1);
    let Object::Paragraph(p1_2, _) = &view5.inner[1].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(p1_2.spans[0].content, "green");

    // Click s2 (Cycle) -> index 2 ("blue")
    let action_cycle_2 = p1_2.spans[0].action.clone().expect("action");
    game.inner.handle_action(action_cycle_2).expect("action");
    let view6 = game.view().expect("view should succeed");
    assert_eq!(game.context.cycle_idx, 2);
    let Object::Paragraph(p1_3, _) = &view6.inner[1].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(p1_3.spans[0].content, "blue");

    // Click s2 (Cycle) -> cycles back to index 0 ("red")
    let action_cycle_3 = p1_3.spans[0].action.clone().expect("action");
    game.inner.handle_action(action_cycle_3).expect("action");
    let view7 = game.view().expect("view should succeed");
    assert_eq!(game.context.cycle_idx, 0);
    let Object::Paragraph(p1_4, _) = &view7.inner[1].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(p1_4.spans[0].content, "red");

    // Click s3 (bracketed Stop) -> index 1 ("two")
    let action_bracket = p2.spans[0].action.clone().expect("action");
    game.inner.handle_action(action_bracket).expect("action");
    let view8 = game.view().expect("view should succeed");
    assert_eq!(game.context.bracket_idx, 1);
    let Object::Paragraph(p2_2, _) = &view8.inner[2].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(p2_2.spans[0].content, "two");
    assert!(p2_2.spans[0].action.is_none());
}

#[derive(Clone, Debug, Default)]
struct VarState {
    chosen: String,
}

#[ifengine::ifview]
fn page_alts_var(state: &mut VarState) {
    let items = ["apple", "banana", "cherry"];
    let s = alts!(items, Cycle, |idx| {
        state.chosen = items[idx].to_string();
    });
    ps!(s);
}

#[test]
fn test_alts_with_variable_expression() {
    let mut game = ifengine::Game::new_with_page("page_alts_var", page_alts_var);
    let view = game.view().expect("view should succeed");

    assert_eq!(game.context.chosen, "apple");

    let Object::Paragraph(p, _) = &view.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(p.spans[0].content, "apple");

    let action = p.spans[0].action.clone().expect("action");
    game.inner.handle_action(action).expect("click");
    let view2 = game.view().expect("view should succeed");

    assert_eq!(game.context.chosen, "banana");
    let Object::Paragraph(p2, _) = &view2.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(p2.spans[0].content, "banana");
}

#[ifengine::ifview]
fn page_stop_and_count_test(_: &mut ()) {
    use ifengine::elements::{count, p};
    let a = alts!(["Click once", "Done"], Stop);
    let c = count!(|n: u64| format!("clicks: {n}"));
    p!(a, " and ", c);
}

#[test]
fn test_stop_drops_action_and_count_clicks() {
    let mut game =
        ifengine::Game::new_with_page("page_stop_and_count_test", page_stop_and_count_test);
    let view = game.view().expect("view should succeed");

    let Object::Paragraph(line, _) = &view.inner[0].object else {
        panic!("expected Object::Paragraph");
    };

    // Initial:
    assert_eq!(line.spans[0].content, "Click once");
    assert!(line.spans[0].action.is_some());
    assert_eq!(line.spans[1].content, " and ");
    assert_eq!(line.spans[2].content, "clicks: 0");
    assert!(line.spans[2].action.is_some());

    // 1st click on count:
    let action_c = line.spans[2].action.clone().expect("count action");
    game.inner.handle_action(action_c).expect("handle count action");
    let view2 = game.view().expect("view 2");
    let Object::Paragraph(line2, _) = &view2.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(line2.spans[2].content, "clicks: 1");

    // 2nd click on count:
    let action_c2 = line2.spans[2].action.clone().expect("count action");
    game.inner.handle_action(action_c2).expect("handle count action");
    let view3 = game.view().expect("view 3");
    let Object::Paragraph(line3, _) = &view3.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(line3.spans[2].content, "clicks: 2");

    // Click alts (Stop):
    let action_a = line3.spans[0].action.clone().expect("alts action");
    game.inner.handle_action(action_a).expect("handle alts action");
    let view4 = game.view().expect("view 4");
    let Object::Paragraph(line4, _) = &view4.inner[0].object else {
        panic!("expected Object::Paragraph");
    };
    assert_eq!(line4.spans[0].content, "Done");
    assert!(line4.spans[0].action.is_none());
}

