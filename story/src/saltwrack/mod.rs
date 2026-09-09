use std::{borrow::Cow, collections::HashSet};

pub mod chap1;
pub mod chap1d;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Walker {
    #[default]
    None,
    A,
    T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Address {
    #[default]
    Sen,
    Ammar,
    Interpreter,
}

impl Address {
    pub fn as_str(&self) -> &'static str {
        match self {
            Address::Sen => "sen",
            Address::Ammar => "ammar",
            Address::Interpreter => "Interpreter",
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct State {
    // Backwards-compatible fields
    pub myname: String,
    pub c1: Companion,
    pub c2: Companion,
    pub days: usize,
    pub rations: usize,
    pub miles: usize,

    // Narrative & system state
    pub address: Address,
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

    pub part1: Part1,
}

impl Default for State {
    fn default() -> Self {
        Self {
            myname: "sen".to_string(),
            c1: Companion::default(),
            c2: Companion::default(),
            days: 0,
            rations: 0,
            miles: 0,
            address: Address::Sen,
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
    fn test_sim() {
        let game = new();
        let _sim = game.simulate(|s| s.depth <= 20);
        dbg!(&_sim);
    }
}
