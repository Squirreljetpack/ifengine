use leptos::prelude::*;

use super::app_impl::StoryApp;
use crate::story;

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

cfg_if::cfg_if! {
    if #[cfg(feature = "test")] {
        pub fn extract_header(_game: &story::Game) -> Vec<String> {
            vec![]
        }
    } else {
        pub fn extract_header(game: &story::Game) -> Vec<String> {
            if game.context.miles != 0 {
                vec![
                    format!("Day: {}", game.context.days),
                    format!("Miles: {}", game.context.miles),
                    format!("Rations: {}", game.context.rations),
                ]
            } else {
                vec![]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
