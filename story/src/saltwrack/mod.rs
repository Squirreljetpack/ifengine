use std::{borrow::Cow, collections::HashSet};

pub(crate) mod chap1d;

pub type Game = ifengine::Game<State>;
pub fn new() -> Game {
    ifengine::Game!(chap1::p1)
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct State {
    pub myname: String,
    pub c1: Companion,
    pub c2: Companion,
    pub days: usize,
    pub rations: usize,
    pub miles: usize,

    pub part1: Part1
}

// Note: all the Into's on str assignment are annoying, maybe derive setters?
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Companion {
    name: Cow<'static, str>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Part1 {
    seen: HashSet<String>
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim() {
        let game = new();
        let _sim = game.simulate(|s| {
            s.depth > 13
        });
        dbg!(&_sim);
    }
}