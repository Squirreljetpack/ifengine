use std::{borrow::Cow, collections::HashSet};

pub mod chap1;
pub mod chap1d;
pub mod chap2;
pub mod chap3;

pub type Game = ifengine::Game<State>;
pub fn new() -> Game {
    ifengine::Game!(chap1::p1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Oracle {
    #[default]
    None,
    V,
    S,
    Dead,
}

impl Oracle {
    pub fn is_dead(&self) -> bool {
        matches!(self, Oracle::Dead)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Walker {
    #[default]
    None,
    A,
    T,
    Dead,
}
impl Walker {
    fn p(self) -> &'static str {
        match self {
            Walker::A => "He",
            _ => "She",
        }
    }
}

impl Walker {
    pub fn is_dead(&self) -> bool {
        matches!(self, Walker::Dead)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct State {
    pub days: usize,
    pub rations: usize,
    pub miles: usize,

    // Narrative & system state
    pub addr: String,
    pub no_interpreter: bool,
    pub oracle: Oracle,
    pub relation_oracle: i32,
    pub walker: Walker,
    pub relation_walker: i32,
    pub seen_eyes: bool,
    pub fuel: usize,
    pub fair_game: bool,
    pub dread: i32,
    pub omen: u32,
    pub north: usize,
    pub lateral: usize,
    pub crew: usize,
    pub checkpoint: usize,
    pub half_rations: bool,
    pub starving: bool,
    pub returning: bool,
    pub bad_air: bool,
    pub used_backstory: bool,
    pub used_vehicle_thought: bool,
    pub used_1w_travel_ques: bool,
    pub used_1w_city_ques: bool,
    pub used_dead_walkers: bool,
    pub used_scavenging: bool,
    pub used_art: bool,
    pub used_wick: bool,
    pub used_history_ques: bool,
    pub used_firmament_convo: bool,
    pub used_occipit: bool,
    pub used_first_dream: bool,
    pub used_spire_forest: bool,
    pub journal_nav: bool,
    pub pool_death: bool,

    pub part1: Part1,
}

impl Default for State {
    fn default() -> Self {
        Self {
            days: 0,
            rations: 0,
            miles: 0,
            addr: "sen".to_string(),
            no_interpreter: false,
            oracle: Oracle::None,
            relation_oracle: 20,
            walker: Walker::None,
            relation_walker: 20,
            seen_eyes: false,
            fuel: 0,
            fair_game: false,
            dread: 0,
            omen: 50,
            north: 0,
            lateral: 0,
            crew: 3,
            checkpoint: 0,
            half_rations: false,
            starving: false,
            returning: false,
            bad_air: false,
            used_backstory: false,
            used_vehicle_thought: false,
            used_1w_travel_ques: false,
            used_1w_city_ques: false,
            used_dead_walkers: false,
            used_scavenging: false,
            used_art: false,
            used_wick: false,
            used_history_ques: false,
            used_firmament_convo: false,
            used_occipit: false,
            used_first_dream: false,
            used_spire_forest: false,
            journal_nav: false,
            pool_death: false,
            part1: Part1::default(),
        }
    }
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Companion {
    pub name: Cow<'static, str>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Part1 {
    pub seen: HashSet<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_chap1() {
        let game = new();
        let sim = game.simulate(|s| s.depth <= 20);
        assert!(!sim.runs.is_empty());
    }

    #[test]
    fn test_sim_chap2() {
        let game = ifengine::Game!(chap2::day_four);
        let sim = game.simulate(|s| s.depth <= 25);
        assert!(!sim.runs.is_empty());
    }

    #[test]
    fn test_sim_chap3() {
        let game = ifengine::Game!(chap3::day_eight);
        let sim = game.simulate(|s| s.depth <= 20);
        assert!(!sim.runs.is_empty());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_saltwrack_serialization() {
        let mut game = new();
        let _ = game.view().expect("view should render");
        game.context.miles = 100;
        game.context.rations = 5;

        let json = serde_json::to_string(&game).expect("serde_json serialize failed");
        println!("Saltwrack JSON size: {} bytes", json.len());

        let bincode_bytes = bincode::serialize(&game).expect("bincode serialize failed");
        println!("Saltwrack Bincode size: {} bytes", bincode_bytes.len());

        let postcard_bytes = postcard::to_allocvec(&game).expect("postcard serialize failed");
        println!("Saltwrack Postcard size: {} bytes", postcard_bytes.len());

        let deserialized_postcard: Game =
            postcard::from_bytes(&postcard_bytes).expect("postcard deserialize failed");
        assert_eq!(deserialized_postcard.context.miles, 100);
        assert_eq!(deserialized_postcard.context.rations, 5);
    }
}
