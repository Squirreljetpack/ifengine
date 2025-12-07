pub mod chap1;

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
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Companion {
    name: String,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim() {
        let game = new();
        let sim = game.simulate(|s| {
            s.depth > 6
        });
        dbg!(&sim);
    }
}