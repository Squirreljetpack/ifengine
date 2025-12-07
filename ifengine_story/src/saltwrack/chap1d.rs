use ifengine::{elements::p, ifview, tun};

use crate::State;

#[ifview]
pub fn _walker(s: &mut State) {
    p!(
        "The proper term of address for a saltwalker is Sel. Without them, no trade would be possible; no transregional communication; no travel. They were the first to breach the wrack, the first to learn its ways. Saltwalker culture may seem superstitious or crude to outsiders. It developed out of necessity, during the apocalypse."
    );

    p!(
        "The first salt snow, an inexplicable deathly miracle, occurred 239 years ago. Its effects were catastrophic: groundwater leaching, dead briny seas, the end of entire ecosystems. The earth's albedo raised, and its carbon diminished as though it were being siphoned. A swift ice age settled. By the time salt no longer sifted from the sky, six harrowed and desperate city-states remained in this corner of the world, isolated by a stretch of hostile white wasteland. Hearth, Clay, Noble, Wick, Firmament, and Rye. ",
        tun!("You recall their names even now in the format of a children's song.")
    );
}

#[ifview]
pub fn _interpreter(s: &mut State) {
    p!(
        "This is your profession, so you ought to know what a good one is. Where a walker interprets the land and an oracle interprets dreams, they interpret the structures of life itself. With scalpel and microscope, scientists like you unravel the biologies of the wrack, facing the mystery of this harsh and frozen world."
    );

    p!(
        "It was said, long ago, that the companions of some creator-deity were interpreters: they named the myriad creatures, dissected newly-made organ systems, tended carefully to the gardens of the heavens. Most people don't believe in gods anymore. ",
        tun!("Most people don't believe in gods anymore.")
    );
}
