#[allow(unused_imports)]
use ifengine::elements::{ChoiceVariant::*, click, dp, p, ps};
use ifengine::{
    back, elements::{choice, h, mchoice}, ifview, link, utils::MaskExt
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
pub fn p3(s: &mut State) {
    ps!(
        "You turn back to the clerk sitting across the desk from you, over piles of slightly crumpled paper. Her hands are stained with ink. Her voice is hoarse, as though she has recently been ill.",
        "Oh—what would you prefer to be addressed as?"
    );

    if let Some(o) = dp!(r#"
    [["Sen."]]  The neutral honorific of respect: a fine choice for any purpose.
    [["Ammat."]]  An honorific translating to “sibling”, common among egalitarians, communalists, and anarchists.
    [["Interpreter."]]  The title of your position, an honorific conveying pride in your skills."#)
    {
        s.myname = o.into();
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

    if mchoice! {
        s.c1.name.is_empty().then_some(link!("the first oracle", _oracle_1)),
        s.c1.name.is_empty().then_some(link!("the second oracle", _oracle_2)),
        s.c2.name.is_empty().then_some(link!("the first saltwalker", _walker_1)),
        s.c2.name.is_empty().then_some(link!("the second saltwalker", _walker_2)),
        (!s.part1.seen.contains("interpreter_2")).then_some(link!("the second interpreter", _interpreter_2))
    }.all() {
        switch!(p6)
    }
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
pub fn _oracle_1(s: &mut State) {
    p!(
        "They are a slight figure, watching you from behind round silver spectacles without meeting your gaze. Their features are angular and delicate. They have icily pale eyes, and their hair is an odd shade of dark grey; perhaps they come from Firmament. Their clothing is drab, neat, and unassuming save for a single drop of dried blood on their collar."
    );

    // until ! becomes a type, we need this last element
    choice! {
        link!("select the first oracle") => {
            s.c1.name = "Danil";
            switch!();
            ""
        },
        back!("consider otherwise")
    };
}

#[ifview]
pub fn _oracle_2(s: &mut State) {
    p!(
        "They are tall, brown-skinned, clad in layers of faded floral-patterned fabric; the overall impression is of striking elegance, despite their dishevelment. Their long hair is tied back in a thick braid. The skin of their arms is inked with spiralling, oddly precise sigils, like blueprints for an unknown mechanism. They gaze off into some dreamy distance, eyes wide and dark and utterly calm."
    );

    choice! {
        link!("select the second oracle") => {
            s.c1.name = "Aron";
            switch!();
            ""
        },
        back!("consider otherwise")
    };
}

#[ifview]
pub fn _walker_1(s: &mut State) {
    p!(
        "He seems every bit a man of Hearth, with his warm dark skin and tightly coiled hair. He must be the oldest person in the room; there are deep wrinkles around his eyes, and his wiry beard is mostly grey. He looks almost skittish. You notice little glinting pendants wired onto his clothing: the saltwalker waysign sigils, you think. You can’t tell what they mean."
    );

    choice! {
        link!("select the first walker") => {
            s.c2.name = "Ego";
            switch!();
            ""
        },
        back!("consider otherwise")
    };
}

#[ifview]
pub fn _walker_2(s: &mut State) {
    p!(
        "She is plumply muscular, short-haired, with moles scattered over her pale skin. Her hands are cut by the crossing marks of scars. She wears a selection of knives openly. Her eyes are concealed by a glassy black helm whose purpose is arcane to you, but her body language seems friendly and energetic."
    );

    choice! {
        link!("select the second walker") => {
            s.c2.name = "Naim";
            switch!();
            ""
        },
        back!("consider otherwise")
    };
}

#[ifview]
pub fn _interpreter_2(s: &mut State) {
    if dp!(r#"
He barely glances at you; he’s preoccupied with turning over some glass model, a green tangle that looks like it might represent the inside of a cell. As you look him over, the functionary notices your attention, and her brow furrows.

“Interpreter?” She’s addressing him. “I thought you—it was agreed you weren’t going to accompany the expedition. We’ve only enough resources for three.” He scoffs, not loudly. “So it’s settled? Rather than another naturalist, you would assign some mystic to my colleague. Thereby preventing any productive discussion in the field, where the observational work of two trained minds would be most valuable. Hardly a scientific expedition, if you ask me.”

The clerk seems weary rather than angered as she begins: “Interpreter, you are aware that given the nature of the salt wrack, the Society’s subcouncil has determined—“ “Spare me.” He gives you a sympathetic grimace as he [[leaves.]]"#).is_some()
    {
        s.part1.seen.insert("interpreter_2");
        switch!()
    }
}

#[ifview]
pub fn p6(s: &mut State) {
    ps!(
        "Her smile widens to a grin. She reaches out to shake your hand; her grip is brief and fever-warm. The electric light in the room glints eerily off her helm. It reminds you of the eyes of a biting fly.",
        link!("And so your crew is selected.", p7)
    );
}

#[ifview]
pub fn p7(s: &mut State) {
    p!(
        "In the following weeks, during the muddle of planning, you don't see much of your new partners. Of course you'll have to share responsibility later on, and rely on your companions, but you can't help thinking of it as your expedition. After all, so many of the decisions are falling to you.",
        "How many days' worth of supplies will you pack? The trip is scheduled to take forty days at most, but it's likely your timing will be off. If you return early, surplus food and fuel will weigh you down. If you return late… you'll have to fend for yourself."
    );

    choice! {
        click!("40 days", {
            s.rations = 400;
            switch!(p8);
        }),
        click!("50 days", {
            s.rations = 500;
            switch!(p8);
        }),
    };
}

#[ifview]
pub fn p8(_: &mut State) {
    if dp!(
        "Some of your time is taken up by organizing supplies, some by being warned. You’ve taken part in a few short expeditions, years ago: but not far from the edges of the city you lived in then, and none longer than a week. This is different. Some have tried, and failed, to reach the heart of the salt wrack. There is a very real chance that you too will never return. You have no doubt your companions are preparing in whatever ways they see fit.

You pore over maps and revisit the Observational Society’s collections, examining microscope slides and desiccated specimens of saltgrown oddities. Even after so much study, you can’t know what to expect from the unknown species and phenomena you’ll witness so far north.

When at last you three meet again, it is in a high-raftered warehouse whose vast doors open northward. The walls are stained with salt and stranger compounds. You clamber into your [[vehicle]], accompanied by your companions."
    ).is_some() {
        switch!(p9)
    }
}


#[ifview]
pub fn p9(s: &mut State) {
    if dp!(
        "This machine, too, is experimental. The latest innovation from Hearth’s engineers. A great metal beast, you think. Quadrupedal, with claws to hook into uneven ground. Wheels wouldn’t be of use in the salt wrack. The legs support a rectangular chamber with seats and room for cargo, piled with various supplies, open to the air but shielded in front. Despite the facelessness of the mechanism, it’s undeniably designed like an animal; the impression is only heightened when you set off, and it begins to walk with a [[steady prowling stride]]."
    ).is_some() {
        s.miles = 40;
        switch!(p10)
    }
}

#[ifview]
pub fn p10(s: &mut State) {
    ps!(
        "There is no boundary between Hearth, your everywhere-place—the familiar place of dwelling—and the forsaken place outside of it, surrounding it. There is no line, not even a fading gradient. Your vehicle lopes over grey frozen heath for a few dozen miles. Soon, there is a thin film of ash or ashlike material on the ground..",
        link!("The sun is a glowing white spot behind the clouds.", p11)
    );


}


#[ifview]
pub fn p11(s: &mut State) {
    ps!(
        "It is summer, and a few living things cling to the ground between patches of rime. Low tundra plants with ghostlike buds. Lichen-stains on crags and boulders. All known and catalogued; you are in search of deeper secrets."
        ,
        "Your colleagues don’t talk much at first. The oracle sits hunched beside you, fidgeting occasionally with their glasses or a spare pencil. The saltwalker holds onto your maps and compasses, for now. Her experience will be useful for at least a few hundred miles north. Beyond that, few travel, even walkers. There’s no reason to go so far beyond civilization. Besides expensive, fatal curiosity.",
        "She seems more willing to chat. You didn’t exactly get to know your colleagues before you were sent out here. You could ask her…");

        if mchoice!(
            link!("“Which city are you from?”"),
            link!("“What’s it like for you—driving a machine instead of walking?”"),
            link!("“How far have you travelled?”"),
            link!("don’t bother her with questions")
        ).any() {
            switch!(p12)
        }
    }


    #[ifview]
    pub fn p12(s: &mut State) {
        dp!(
            "She smirks. “Which city? Why don’t you guess?”",
            "It’s hard to say. Skin as pale as hers is uncommon in Hearth and Rye, but none of the city-states were ever composed of only one ethnicity. Without cultural tells, you can’t hazard a guess. And she seems pleased to have stumped you.",
            "Not long afterwards, she eases the vehicle to a stop. The sun is slanting low, and you’ll need time to set up camp. The first day of travel is over. You’re not sure quite what to expect from [[the first night]]."
        );
    }