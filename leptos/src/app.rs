use leptos::prelude::*;

use super::app_impl::{RawHeaderItem, StoryApp};
use crate::story;

#[component]
pub fn App() -> impl IntoView {
    let initial_game = story::new();

    view! {
        <StoryApp
            game=initial_game
            header_extractor=Some(extract_header as fn(&story::Game) -> [Vec<RawHeaderItem<story::State>>; 3])
        />
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "test")] {
        pub fn extract_header(_game: &story::Game) -> [Vec<RawHeaderItem<story::State>>; 3] {
            [vec![], vec![], vec![]]
        }
    } else if #[cfg(feature = "chant")] {
        pub fn extract_header(game: &story::Game) -> [Vec<RawHeaderItem<story::State>>; 3] {
            let header = game.context.header();
            let left = header.first().cloned().map(RawHeaderItem::Text).into_iter().collect();
            let center = header.get(1).cloned().map(RawHeaderItem::Text).into_iter().collect();

            let mut right: Vec<RawHeaderItem<story::State>> = header
                .get(2)
                .cloned()
                .map(RawHeaderItem::Text)
                .into_iter()
                .collect();
            if game.context.has_inventory() {
                right.push(RawHeaderItem::Action {
                    label: "Inventory".to_string(),
                    handler: toggle_inventory,
                });
            }

            [left, center, right]
        }

        fn toggle_inventory(game: &mut story::Game) {
            game.context.inventory_open = !game.context.inventory_open;
        }
    } else {
        pub fn extract_header(game: &story::Game) -> [Vec<RawHeaderItem<story::State>>; 3] {
            if game.context.miles != 0 {
                [
                    vec![RawHeaderItem::Text(format!("Day: {}", game.context.days))],
                    vec![RawHeaderItem::Text(format!("Miles: {}", game.context.miles))],
                    vec![RawHeaderItem::Text(format!("Rations: {}", game.context.rations))],
                ]
            } else {
                [vec![], vec![], vec![]]
            }
        }
    }
}
