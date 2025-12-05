#[allow(unused_imports)]
use ifengine::elements::{ChoiceVariant::*, click, dp, p, ps};
use ifengine::{
    elements::{choice, h, mchoice},
    ifview, link,
};
use ifengine::{switch, tun};

use crate::saltwrack::State;

#[ifview]
pub fn p1(s: &mut State) {
    ps!(
        "If summer meant heat, this ground would thaw. The thin soil would flourish; the wastes beyond the city would be green.",
        "You gaze out to the east, past the old ornate window of the Observational Society. The sun glares on dead white. South, the city's low familiar skyline begins, the buildings hunched as though they fear the sky. Soon you will leave this place.",
        link!("North lies the salt wrack. North is where you will go.", p2)
    );
}

#[ifview]
pub fn p2(s: &mut State) {
    h!("SALTWRACK", 3);
    p!(link!("BEGIN", p3));
}

#[ifview]
pub fn p3(state: &mut State) {
    ps!(
        "You turn back to the clerk sitting across the desk from you, over piles of slightly crumpled paper. Her hands are stained with ink. Her voice is hoarse, as though she has recently been ill.",
        "Oh—what would you prefer to be addressed as?"
    );

    if let Some(s) = dp!(r#"
    [["Sen."]]  The neutral honorific of respect: a fine choice for any purpose.
    [["Ammat."]]  An honorific translating to “sibling”, common among egalitarians, communalists, and anarchists.
    [["Interpreter."]]  The title of your position, an honorific conveying pride in your skills."#)
    {
        state.myname = s.into();
        switch!(p4)
    }
}

#[ifview]
pub fn p4(s: &mut State) {
    p!("The functionary nods. “I'll tell the others as much, sen.”");

    p!(
        "It's taken months to persuade the Observational Society to sign off on your expedition. But now, most of the obstacles are out of the way, and concrete reality is setting in: there are only a few weeks left before your departure. You've been assured that the Society's clerk will see to the budgeting. “All expenses have been approved,” she reminds you. “And in case something happens to one of you out there… well, you're undertaking this expedition with a full awareness of the risks. As are your colleagues.” She gives you a half-smile. “I hear your candidates are a scholar or two, a couple of saltwalkers, and… an artist. Oh, don't look at me like that. You know what oracles are like. It's a wonder we managed to find two who wanted to be part of this.”"
    );

    p!(
        "Reassuring. ",
        link!("It's time to see who you'll be working with.", p5)
    );
}

#[ifview]
pub fn p5(s: &mut State) {
    p!(
        "You are shown to a small gathering-room wallpapered in grubby tapestry. Weak light filters through its salt-crusted windows. Five unfamiliar people sit around the table—actually, you're sure you've seen some of them in passing, in the corridors of the Observational Society. But it doesn't seem like you'll have a chance to talk to them individually before making your choices."
    );

    p!(
        "It would be folly to go without a ",
        tun!("walker", _walker),
        ". You've been pressured to select an oracle as your other colleague. But as you scan over the faces, you note the unexpected presence of another ",
        tun!("interpreter", _interpreter),
        "."
    );

    mchoice! {
        s.c1.name.is_empty().then_some(link!("the first oracle")),
        s.c1.name.is_empty().then_some(link!("the second oracle")),
        s.c2.name.is_empty().then_some(link!("the first saltwalker")),
        s.c2.name.is_empty().then_some(link!("the second saltwalker")),
        link!("the second interpreter"),
    };
}

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

#[ifview]
pub fn p6(s: &mut State) {
    p!(
        "In the following weeks, during the muddle of planning, you don't see much of your new partners. Of course you'll have to share responsibility later on, and rely on your companions, but you can't help thinking of it as your expedition. After all, so many of the decisions are falling to you.",
        "How many days' worth of supplies will you pack? The trip is scheduled to take forty days at most, but it's likely your timing will be off. If you return early, surplus food and fuel will weigh you down. If you return late… you'll have to fend for yourself."
    );

    choice! {
        click!("40 days", {
            s.days = 40;
            switch!(p2);
        }),
        click!("50 days", {
            s.days = 50;
            switch!(p2);
        }),
    };
}
// lines!
// Takes a stromg "[[text]] abcdef", and an optional closure |x| { }, and runs the closure on the highlighted text

#[ifview]
pub fn p7(_: &mut State) {
    p!(
        "Some of your time is taken up by organizing supplies, some by being warned. You've taken part in a few short expeditions, years ago: but not far from the edges of the city you lived in then, and none longer than a week. This is different. Some have tried, and failed, to reach the heart of the salt wrack. There is a very real chance that you too will never return. You have no doubt your companions are preparing in whatever ways they see fit."
    );
}


