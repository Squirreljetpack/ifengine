use ifengine::{
    elements::{p, tun},
    ifview,
};

use crate::saltwrack::State;

#[ifview]
pub fn _about(_s: &mut State) {
    p!(
        "SALTWRACK is a doomed expedition simulator/post-post-apocalyptic hypertext horror novel. You play as a scientist in a crew of three explorers investigating a land scoured by disaster. Things are not likely to go well for you."
    );
    p!(
        "SALTWRACK contains strong horror themes and may not be suitable for all audiences. Descriptions of death, violence, self-injury, paranoia, sickness, and starvation may be encountered, depending on choice and chance. Reader discretion is advised."
    );
    p!(
        "This story was created by Henry Kay Cecchini (they/them), using Twine's Chapbook format. It is an effort of 6 years and around 78,000 words of prose and programming combined."
    );
    p!(
        "Special thanks to Davey for providing testing and feedback! And my sincere apologies for any bugs that remain."
    );
    p!(tun!("Go back"));
}

#[ifview]
pub fn _walker(_s: &mut State) {
    p!(
        "The proper term of address for a saltwalker is Sel. Without them, no trade would be possible; no transregional communication; no travel. They were the first to breach the wrack, the first to learn its ways. Saltwalker culture may seem superstitious or crude to outsiders. It developed out of necessity, during the apocalypse."
    );
    p!(
        "The first salt snow, an inexplicable deathly miracle, occurred 239 years ago. Its effects were catastrophic: groundwater leaching, dead briny seas, the end of entire ecosystems. The earth's albedo raised, and its carbon diminished as though it were being siphoned. A swift ice age settled. By the time salt no longer sifted from the sky, six harrowed and desperate city-states remained in this corner of the world, isolated by a stretch of hostile white wasteland. Hearth, Clay, Noble, Wick, Firmament, and Rye. ",
        tun!("You recall their names even now in the format of a children's song.")
    );
}

#[ifview]
pub fn _oracle(_s: &mut State) {
    p!(
        "Oracles are a strange class of people: those whose minds are touched by something outside the usual sphere. They are often androgynes, usually asocial or inclined towards solitude. They possess eerie abilities, unorthodox ways of thought, and more senses than humans generally have. Vivid, prophetic dreams, or visions of impossible shapes, or perpetual knowledge of where the poles are. Some oracles are highly respected researchers and theorists. Some are the object of cults. ",
        tun!("Some burn themselves alive.")
    );
}

#[ifview]
pub fn _interpreter(s: &mut State) {
    s.no_interpreter = true;
    p!(
        "This is your profession, so you ought to know what a good one is. Where a walker interprets the land and an oracle interprets dreams, they interpret the structures of life itself. With scalpel and microscope, scientists like you unravel the biologies of the wrack, facing the mystery of this harsh and frozen world."
    );
    p!(
        "It was said, long ago, that the companions of some creator-deity were interpreters: they named the myriad creatures, dissected newly-made organ systems, tended carefully to the gardens of the heavens. ",
        tun!("Most people don't believe in gods anymore.")
    );
}

#[ifview]
pub fn _backstory(_s: &mut State) {
    p!(
        "Firmament has no central governing body. No law guaranteed you the right to undertake expeditions in pursuit of your work. After years of shared research, the interpreters' collective denied you the chance to travel to the origin of the cataclysm. They termed it a waste of the city's resources, and of your life besides. Your work was criticized harshly for its lack of material relevance, its idealistic indulgence. It did not contribute to the state's survival or improve its conditions. It was misguided, or so the idea went, and you would do better to drop it."
    );
    p!(
        "But you were in contact with interpreters from other cities, and your work excited them. Someone in the Observational Society pulled strings to sponsor you. And so Hearth accepted you, welcomed you. Hearth wanted you, when Firmament was indifferent."
    );
    p!(
        "You were conflicted; you could hardly have been otherwise. To leave your city was to leave behind your home, your loyalty, everyone you knew. The journey to Hearth was long and difficult, and the city seemed at first unimpressive, even shabby. You couldn't stop seeing the contrast between university grounds, or the ornate buildings of the Council, and common housing."
    );
    p!(
        "But then the chair of the Observational Society, eager to make your acquaintance, invited you to a formal lunch. That meal is still linked in your mind with the first sights of Hearth, with Hearth as a whole, actually. Crisp bread, soft colorless melon. Some sort of warm savory beverage. Your new city seemed temperate, expansive, for a time."
    );
    p!(tun!("Continue..."));
}

#[ifview]
pub fn _the_vehicle(s: &mut State) {
    s.used_vehicle_thought = true;
    p!(
        "In the years after the salt snow, there were no such machines. Saltwalkers, and those they accompanied, travelled on foot. Mechanics from the city of Noble were the first to create a vessel that could carry passengers over the wrack. But there's a reason why saltwalkers are still walkers. The wrack does strange things to machinery: altered circuits, inexplicable failures, simple chemical corrosion. Engineers are never entirely dissuaded by this, and every year brings news of some attempt to solve the problem of travel once and for all. The vehicle that carries you is as experimental as your entire goal."
    );
    p!(
        "Further north, on the great glacial ice sheet, you'll truly prove whether or not this model of machine can handle all the variable conditions of this wasteland. At least one mountain range stretches between you and the objective. In the worst case, you can make part of the journey on foot, the traditional way."
    );
    p!(
        "Your expedition is supplied with far more fuel than you'll actually use. That, at least, will not be a point of failure."
    );
    p!(tun!("But why even think about failing?"));
}
