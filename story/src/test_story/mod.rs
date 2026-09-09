pub mod chap1;

pub type Game = ifengine::Game<()>;
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
}
