pub mod chap1;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct State {
    pub job: Option<String>,
    pub miles: usize,
    pub days: usize,
    pub rations: usize,
}

pub type Game = ifengine::Game<State>;
pub fn new() -> Game {
    ifengine::Game!(chap1::rainy_day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_runs() {
        let mut game = new();
        let view = game.view().expect("failed to render rainy_day");
        assert!(view.pageid.0.ends_with("rainy_day"));
    }

    #[test]
    fn test_embedded_weather_station_choice_and_link() {
        use ifengine::view::Object;

        let mut game = new();
        let view = game.view().expect("failed to render rainy_day");

        // Find the embedded Object::Embed
        let embedded_obj = view.inner.iter().find(|obj| matches!(obj, Object::Embed(_)));
        assert!(embedded_obj.is_some(), "weather_station should be embedded");

        let Object::Embed(sub_view) = embedded_obj.unwrap() else { unreachable!() };
        assert_eq!(sub_view.pageid, view.pageid);

        // Verify choice and link inside embedded sub_view
        let has_choice = sub_view.inner.iter().any(|obj| matches!(obj, Object::Choice(..)));
        assert!(has_choice, "embedded sub_view must contain Object::Choice");

        // Find the link to sensor_logs in the embedded sub_view
        let mut link_action = None;
        for obj in &sub_view.inner {
            if let Object::Paragraph(line) = obj {
                for span in &line.spans {
                    if let Some(action) = &span.action {
                        link_action = Some(action.clone());
                    }
                }
            }
        }
        assert!(link_action.is_some(), "embedded sub_view must contain link action");

        // Trigger the link action: navigate to sensor_logs
        game.inner.handle_action(link_action.unwrap()).expect("action should succeed");
        let logs_view = game.view().expect("should render sensor_logs");
        assert!(logs_view.pageid.0.ends_with("sensor_logs"));

        // Find the back button in sensor_logs
        let mut back_action = None;
        for obj in &logs_view.inner {
            if let Object::Paragraph(line) = obj {
                for span in &line.spans {
                    if let Some(action) = &span.action {
                        back_action = Some(action.clone());
                    }
                }
            }
        }
        assert!(back_action.is_some(), "sensor_logs must have a back action");

        // Trigger back action: return to rainy_day
        game.inner.handle_action(back_action.unwrap()).expect("back should succeed");
        let return_view = game.view().expect("should render rainy_day after back");
        assert!(return_view.pageid.0.ends_with("rainy_day"));

        // Now test choice collapsing:
        // Because embedded view.pageid is overwritten with the parent pageid,
        // frontends can dispatch choices directly using the view's own pageid:
        let Object::Embed(sub_view_after_back) = return_view.inner.iter().find(|obj| matches!(obj, Object::Embed(_))).unwrap() else { unreachable!() };
        let choice_key = sub_view_after_back.inner.iter().find_map(|obj| {
            if let Object::Choice(key, _) = obj {
                Some(*key)
            } else {
                None
            }
        }).expect("must have choice key");

        // Dispatch choice 0 ("Check barometer") directly with choice_key:
        game.inner.handle_choice(choice_key, 0);

        // Re-render rainy_day:
        let updated_view = game.view().expect("should render rainy_day with collapsed choice");
        let Object::Embed(updated_sub_view) = updated_view.inner.iter().find(|obj| matches!(obj, Object::Embed(_))).unwrap() else { unreachable!() };

        // Verify the choice COLLAPSED!
        let has_uncollapsed_choice = updated_sub_view.inner.iter().any(|obj| matches!(obj, Object::Choice(..)));
        assert!(!has_uncollapsed_choice, "choice in embedded view should now be collapsed into Paragraph!");

        let has_replacement = updated_sub_view.inner.iter().any(|obj| {
            if let Object::Paragraph(line) = obj {
                line.content().contains("barometric pressure is 982 hPa")
            } else {
                false
            }
        });
        assert!(has_replacement, "collapsed replacement text must be present in embedded view!");
    }

    #[test]
    fn test_cover_story_replace_and_timed_choice() {
        use ifengine::view::Object;

        let mut game = new();
        let view = game.view().expect("failed to render rainy_day");

        // 1. Initial state: verify linkreplace is displayed as a link
        let mut replace_link_action = None;
        for obj in &view.inner {
            if let Object::Paragraph(line) = obj {
                if line.content().contains("wanted criminal") {
                    for span in &line.spans {
                        if span.content.contains("wanted criminal") {
                            replace_link_action = span.action.clone();
                        }
                    }
                }
            }
        }
        assert!(replace_link_action.is_some(), "wanted criminal link must be present initially");

        // Verify that the cover story choices are NOT visible yet
        let has_treasure_hunter = view.inner.iter().any(|obj| match obj {
            Object::Choice(_, choices) => choices.iter().any(|(_, l)| l.content().contains("treasure hunter")),
            Object::Paragraph(l) => l.content().contains("treasure hunter"),
            _ => false,
        });
        assert!(!has_treasure_hunter, "choices should not be visible before link is clicked");
        assert_eq!(game.context.job, None);

        // 2. Click the linkreplace action
        game.inner.handle_action(replace_link_action.unwrap()).expect("action should succeed");
        let view2 = game.view().expect("should render after clicking replace");

        // Verify link was replaced with "Obviously you're not going to tell the truth..."
        let has_obviously = view2.inner.iter().any(|obj| {
            if let Object::Paragraph(line) = obj {
                line.content().contains("Obviously you're not going to tell the truth")
            } else {
                false
            }
        });
        assert!(has_obviously, "replacement paragraph must be visible");

        // Verify choices are now present and have the "in-500" transition class
        let cover_choice = view2.inner.iter().find_map(|obj| {
            if let Object::Choice(key, choices) = obj {
                if choices.iter().any(|(_, l)| l.content().contains("treasure hunter")) {
                    Some((*key, choices.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        }).expect("cover story choice must be present after replace");

        let (choice_key, choice_lines) = cover_choice;
        assert_eq!(choice_lines.len(), 3);

        for (_, line) in &choice_lines {
            // Check that the line or its span contains "in-500"
            let has_in_500 = line.classes.iter().any(|c| c == "in-500")
                || line.spans.iter().any(|s| s.classes.iter().any(|c| c == "in-500"));
            assert!(has_in_500, "each choice option must have class 'in-500'");
        }

        // 3. Select choice 0 ("treasure hunter")
        game.inner.handle_choice(choice_key, 0);
        let view3 = game.view().expect("should render after choosing job");

        // Verify state was updated
        assert_eq!(game.context.job.as_deref(), Some("treasure hunter"));

        // Verify job is displayed after the choice
        let has_job_display = view3.inner.iter().any(|obj| {
            if let Object::Paragraph(line) = obj {
                line.content().contains("Job: treasure hunter")
            } else {
                false
            }
        });
        assert!(has_job_display, "view must display 'Job: treasure hunter' after selection");
    }
}
