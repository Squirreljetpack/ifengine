use leptos::prelude::*;

use super::app_impl::StoryApp;

/// Main Leptos application component for `ifengine`.
///
/// Automatically mounts the active story configured by feature flags (`saltwrack` or `test`),
/// extracting header metadata via [`extract_header`].
#[component]
pub fn App() -> impl IntoView {
    let initial_game = story::new();

    view! {
        <StoryApp
            game=initial_game
            header_extractor=extract_header
        />
    }
}

/// Extracts header statistics (Day, Miles travelled, Rations) from story state if available.
#[cfg(all(feature = "saltwrack", not(feature = "test")))]
pub fn extract_header(game: &story::Game) -> Vec<String> {
    if game.context.miles != 0 {
        vec![
            format!("Day: {}", game.context.days),
            format!("Miles travelled: {}", game.context.miles),
            format!("Rations: {}", game.context.rations),
        ]
    } else {
        vec![]
    }
}

/// Extracts header statistics when using `test_story` (empty status header).
#[cfg(any(feature = "test", not(feature = "saltwrack")))]
pub fn extract_header(_game: &story::Game) -> Vec<String> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_resolution() {
        let mut game = story::new();
        let view = game.view().expect("should render initial view");

        #[cfg(all(feature = "saltwrack", not(feature = "test")))]
        {
            assert!(
                view.pageid.0.contains("saltwrack"),
                "expected saltwrack story, got: {}",
                view.pageid.0
            );
            let header = extract_header(&game);
            assert!(header.is_empty());
        }

        #[cfg(any(feature = "test", not(feature = "saltwrack")))]
        {
            assert!(
                view.pageid.0.contains("rainy_day"),
                "expected test_story (rainy_day), got: {}",
                view.pageid.0
            );
            let header = extract_header(&game);
            assert!(header.is_empty());
        }
    }
}
