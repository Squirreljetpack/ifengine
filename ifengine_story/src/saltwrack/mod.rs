use serde::{Deserialize, Serialize};

pub mod chap1;

pub type Game = ifengine::Game<State>;
pub fn new() -> Game {
    ifengine::Game!(chap1::p1)
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub myname: String,
    pub c1: Companion,
    pub c2: Companion,
    pub days: usize,
    pub rations: usize,
    pub miles: usize,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Companion {
    name: String,
}
