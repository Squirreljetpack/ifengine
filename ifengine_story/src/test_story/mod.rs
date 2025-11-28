pub mod chap1;

pub type Game = ifengine::Game<()>;
pub fn new() -> Game {
    ifengine::Game!(chap1::rainy_day)
}