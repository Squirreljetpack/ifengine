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
}
