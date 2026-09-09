use crate::{
    chap1d::*,
    saltwrack::{Oracle, State, Walker},
};
use ifengine::elements::*;
use ifengine::{ifview, utils::MaskExt};

#[ifview]
pub fn p1(_s: &mut State) {
    ps!(
        "If summer meant heat, this ground would thaw. The thin soil would flourish; the wastes beyond the city would be green.",
        "You gaze out to the east, past the old ornate window of the Observational Society. The sun glares on dead white. South, the city's low familiar skyline begins, the buildings hunched as though they fear the sky. Soon you will leave this place.",
        link!("North lies the salt wrack. North is where you will go.", p2)
    );
}

#[ifview]
pub fn p2(_s: &mut State) {
    h!(
        s!("SALTWRACK"),
        // .cls("center")
        // .style("display", "block")
        // .style("margin-top", "clamp(6rem, 15vh, 16rem)"),
        3
    );
    choice!(tun!("ABOUT", _about), link!("BEGIN", p3));
}

#[ifview]
pub fn p3(state: &mut State) {
    ps!(
        "You turn back to the clerk sitting across the desk from you, over piles of slightly crumpled paper. Her hands are stained with ink. Her voice is hoarse, as though she has recently been ill.",
        r#""Oh—what would you prefer to be addressed as?""#
    );

    let names = [
        "Sen",
        "The neutral honorific of respect: a fine choice for any purpose.",
        "Ammar",
        "An honorific translating to “sibling”, common among egalitarians, communalists, and anarchists.",
        "Interpreter",
        "The title of your position, an honorific conveying pride in your skills.",
    ];

    let mut choices = vec![];
    for i in 0..3 {
        let e = click!(names[i * 2], {
            state.addr = match i {
                0 => "sen".to_string(),
                1 => "ammar".to_string(),
                _ => "Interpreter".to_string(),
            };
            NEXT!(p4)
        });
        choices.push([e, s!(".  ", names[i * 2 + 1])]);
    }
    dchoice!(choices);

    // ------- ALTERNATIVES --------
    // // The whole line is clickable:
    //
    // let choices: Vec<_> = names
    //     .chunks_exact(2)
    //     .map(|x| [link!(x[0]), s!(".  ", x[1])])
    //     .collect();

    // dchoice! { choices,
    //     c => {
    //         state.myname = names[c * 2].to_string();
    //         NEXT!(p4);
    //     }
    // }

    // dchoice!(choices);

    // // Key override:
    //
    // let choices: Vec<_> = names
    //     .chunks_exact(2)
    //     .enumerate()
    //     .map(|(i, x)| [click!((i as u64), x[0]), s!(".  ", x[1])])
    //     .collect();

    // dchoice!(choices);

    // for i in 0..3 {
    //     if read_key!(i).is_some() {
    //         state.myname = names[i as usize * 2].to_string();
    //         NEXT!(p4)
    //     }
    // }
}

#[ifview]
pub fn p4(s: &mut State) {
    p!("The functionary nods. “I'll tell the others as much, {s.addr}.”");

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
        ". You've been pressured to select an ",
        tun!("oracle", _oracle),
        " as your other colleague.",
    );
    if !s.no_interpreter {
        x![
            "But as you scan over the faces, you note the unexpected presence of another ",
            tun!("interpreter", _interpreter),
            "."
        ];
    }

    let need_oracle = s.oracle == Oracle::None;
    let need_walker = s.walker == Walker::None;

    if (s.oracle != Oracle::None && s.walker != Walker::None)
        || mchoice! {
            need_oracle.then_some(link!("the first oracle", view_oracle_1)),
            need_oracle.then_some(link!("the second oracle", view_oracle_2)),
            need_walker.then_some(link!("the first saltwalker", view_walker_1)),
            need_walker.then_some(link!("the second saltwalker", view_walker_2)),
            (!s.no_interpreter).then_some(link!("the second interpreter", view_interpreter_2)),
        }
        .all()
    {
        p!(link!("And so your crew is selected.", crew_selected));
    }
}

#[ifview]
pub fn view_oracle_1(s: &mut State) {
    p!(
        "They are a slight figure, watching you from behind round silver spectacles without meeting your gaze. Their features are angular and delicate. They have icily pale eyes, and their hair is an odd shade of dark grey; perhaps they come from Firmament. Their clothing is drab, neat, and unassuming save for a single drop of dried blood on their collar."
    );

    if s.no_interpreter {
        p!("You note that they seemed to bristle at the other interpreter's remark.");
    }

    choice!(
        link!("select the first oracle", select_oracle_1),
        back!("consider otherwise")
    );
}

#[ifview]
pub fn select_oracle_1(s: &mut State) {
    s.oracle = Oracle::V;
    s.relation_oracle = 20;

    p!(
        "“Yes. Thank you ever so much, Interpreter.” Their tone is unexpectedly fervent, given their carefully controlled appearance. The eyes of the two oracles meet. Some unshared communication seems to pass between them in that glance. The other oracle rests their fingertips on your new partner's wrist, for just a moment."
    );

    if s.walker == Walker::None {
        p!("You still have a walker to choose.");
        choice!(
            link!("the first saltwalker", view_walker_1),
            link!("the second saltwalker", view_walker_2)
        );
    } else {
        p!(link!("And so your crew is selected.", crew_selected));
    }
}

#[ifview]
pub fn view_oracle_2(_s: &mut State) {
    p!(
        "They are tall, brown-skinned, clad in layers of faded floral-patterned fabric; the overall impression is of striking elegance, despite their dishevelment. Their long hair is tied back in a thick braid. The skin of their arms is inked with spiralling, oddly precise sigils, like blueprints for an unknown mechanism. They gaze off into some dreamy distance, eyes wide and dark and utterly calm."
    );

    choice!(
        link!("select the second oracle", select_oracle_2),
        back!("consider otherwise")
    );
}

#[ifview]
pub fn select_oracle_2(s: &mut State) {
    s.oracle = Oracle::S;
    s.relation_oracle = 20;

    p!("They nod. “I will serve as well as I can.” You can't tell what they're thinking.");
    p!(
        "The eyes of the two oracles meet. Some unshared communication seems to pass between them in that glance. The other oracle offers your new partner an uncertain smile."
    );

    if s.walker == Walker::None {
        p!("You still have a walker to choose.");
        choice!(
            link!("the first saltwalker", view_walker_1),
            link!("the second saltwalker", view_walker_2)
        );
    } else {
        p!(link!("And so your crew is selected.", crew_selected));
    }
}

#[ifview]
pub fn view_walker_1(_s: &mut State) {
    p!(
        "He seems every bit a man of Hearth, with his warm dark skin and tightly coiled hair. He must be the oldest person in the room; there are deep wrinkles around his eyes, and his wiry beard is mostly grey. He looks almost skittish. You notice little glinting pendants wired onto his clothing: the saltwalker waysign sigils, you think. You can't tell what they mean."
    );

    choice!(
        link!("select the first walker", select_walker_1),
        back!("consider otherwise")
    );
}

#[ifview]
pub fn select_walker_1(s: &mut State) {
    s.walker = Walker::A;
    s.relation_walker = 20;
    p!(
        "He stands and inclines his head in a very courteous, rather old-fashioned acknowledgement. “It is an honor, {s.addr}.” His voice is solemn; his eyes look sharply into yours. The other saltwalker watches him, her slight smile gone."
    );

    if s.oracle == Oracle::None {
        p!("You still have an oracle to choose.");
        choice!(
            link!("the first oracle", view_oracle_1),
            link!("the second oracle", view_oracle_2)
        );
    } else {
        p!(link!("And so your crew is selected.", crew_selected));
    }
}

#[ifview]
pub fn view_walker_2(_s: &mut State) {
    p!(
        "She is plumply muscular, short-haired, with moles scattered over her pale skin. Her hands are cut by the crossing marks of scars. She wears a selection of knives openly. Her eyes are concealed by a glassy black helm whose purpose is arcane to you, but her body language seems friendly and energetic."
    );

    choice!(
        link!("select the second walker", select_walker_2),
        back!("consider otherwise")
    );
}

#[ifview]
pub fn select_walker_2(s: &mut State) {
    s.walker = Walker::T;
    s.relation_walker = 20;
    s.seen_eyes = false;

    p!(
        "Her smile widens to a grin. She reaches out to shake your hand; her grip is brief and fever-warm. The electric light in the room glints eerily off her helm. It reminds you of the eyes of a biting fly."
    );

    if s.oracle == Oracle::None {
        p!("You still have an oracle to choose.");
        choice!(
            link!("the first oracle", view_oracle_1),
            link!("the second oracle", view_oracle_2)
        );
    } else {
        p!(link!("And so your crew is selected.", crew_selected));
    }
}

#[ifview]
pub fn view_interpreter_2(s: &mut State) {
    s.no_interpreter = true;

    p!(
        "He barely glances at you; he's preoccupied with turning over some glass model, a green tangle that looks like it might represent the inside of a cell. As you look him over, the functionary notices your attention, and her brow furrows."
    );
    p!(
        "“Interpreter?” She's addressing him. “I thought you—it was agreed you weren't going to accompany the expedition. We've only enough resources for three.” He scoffs, not loudly. “So it's settled? Rather than another naturalist, you would assign some mystic to my colleague. Thereby preventing any productive discussion in the field, where the observational work of two trained minds would be most valuable. Hardly a scientific expedition, if you ask me.”"
    );
    p!(
        "The clerk seems weary rather than angered as she begins: “Interpreter, you are aware that given the nature of the salt wrack, the Society's subcouncil has determined—” “Spare me.” He gives you a sympathetic grimace as he ",
        back!("leaves.")
    );
}

#[ifview]
pub fn crew_selected(s: &mut State) {
    p!(
        "In the following weeks, during the muddle of planning, you don't see much of your new partners. Of course you'll have to share responsibility later on, and rely on your companions, but you can't help thinking of it as your expedition. After all, so many of the decisions are falling to you."
    );
    p!(
        "How many days' worth of supplies will you pack? The trip is scheduled to take forty days at most, but it's likely your timing will be off. If you return early, surplus food and fuel will weigh you down. If you return late… you'll have to fend for yourself."
    );

    choice!(
        click!("40 days", {
            s.rations = 240;
            s.fuel = 2;
            s.fair_game = false;
            NEXT!(preparations);
        }),
        click!("50 days", {
            s.rations = 300;
            s.fuel = 3;
            s.fair_game = true;
            NEXT!(preparations);
        })
    );
}

#[ifview]
pub fn preparations(_s: &mut State) {
    p!(
        "Some of your time is taken up by organizing supplies, some by being warned. You've taken part in a few short expeditions, years ago: but not far from the edges of the city you lived in then, and none longer than a week. This is different. Some have tried, and failed, to reach the heart of the salt wrack. There is a very real chance that you too will never return. You have no doubt your companions are preparing in whatever ways they see fit."
    );
    p!(
        "You pore over maps and revisit the Observational Society's collections, examining microscope slides and desiccated specimens of saltgrown oddities. Even after so much study, you can't know what to expect from the unknown species and phenomena you'll witness so far north."
    );
    p!(
        "When at last you three meet again, it is in a high-raftered warehouse whose vast doors open northward. The walls are stained with salt and stranger compounds. You clamber into your ",
        link!("vehicle", vehicle_intro),
        ", accompanied by your companions."
    );
}

#[ifview]
pub fn vehicle_intro(s: &mut State) {
    let cargo_desc = if s.rations >= 300 {
        "heaped with various supplies"
    } else {
        "piled with various supplies"
    };

    p!(
        "This machine, too, is experimental. The latest innovation from Hearth's engineers. A great metal beast, you think. Quadrupedal, with claws to hook into uneven ground. Wheels wouldn't be of use in the salt wrack. The legs support a rectangular chamber with seats and room for cargo, ",
        s!(cargo_desc),
        ", open to the air but shielded in front. Despite the facelessness of the mechanism, it's undeniably designed like an animal; the impression is only heightened when you set off, and it begins to walk with ",
        link!("a steady prowling stride", day1_start),
        "."
    );
}

#[ifview]
pub fn day1_start(s: &mut State) {
    s.days = 1;
    s.dread = 1;
    s.miles = 40;
    s.north = 40;
    s.lateral = 0;
    s.crew = 3;

    p!(
        "There is no boundary between Hearth, your everywhere-place—the familiar place of dwelling—and the forsaken place outside of it, surrounding it. There is no line, not even a fading gradient. Your vehicle lopes over grey frozen heath for a few dozen miles. Soon, there is a thin film of ash or ashlike material on the ground."
    );
    p!(link!(
        "The sun is a glowing white spot behind the clouds.",
        day1_travel
    ));
}

#[ifview]
pub fn day1_travel(s: &mut State) {
    p!(
        "It is summer, and a few living things cling to the ground between patches of rime. Low tundra plants with ghostlike buds. Lichen-stains on crags and boulders. All known and catalogued; you are in search of deeper secrets."
    );
    p!("Your colleagues don't talk much at first.");

    match s.oracle {
        Oracle::S => {
            p!(
                "The oracle sits beside you, watching the bleak landscape. They barely seem to notice you."
            );
        }
        _ => {
            p!(
                "The oracle sits hunched beside you, fidgeting occasionally with their glasses or a spare pencil."
            );
        }
    }

    match s.walker {
        Walker::T => {
            p!(
                "The saltwalker holds onto your maps and compasses, for now. Her experience will be useful for at least a few hundred miles north. Beyond that, few travel, even walkers. There's no reason to go so far beyond civilization. Besides expensive, fatal curiosity."
            );
            p!(
                "She seems more willing to chat. You didn't exactly get to know your colleagues before you were sent out here. You could ask her…"
            );
            choice!(
                link!("“Which city are you from?”", d1_t_city),
                link!(
                    "“What's it like for you—driving a machine instead of walking?”",
                    d1_t_driving
                ),
                link!("“How far have you travelled?”", d1_t_travel),
                link!("don't bother her with questions", d1_no_convo)
            );
        }
        _ => {
            p!(
                "The saltwalker holds onto your maps and compasses, for now. His experience will be useful for at least a few hundred miles north. Beyond that, few travel, even walkers. There's no reason to go so far beyond civilization. Besides expensive, fatal curiosity."
            );
            p!(
                "He seems more willing to chat. You didn't exactly get to know your colleagues before you were sent out here. You could ask him…"
            );
            choice!(
                link!("“You're from Hearth, aren't you?”", d1_a_city),
                link!(
                    "“What's it like for you—driving a machine instead of walking?”",
                    d1_a_driving
                ),
                link!("“How far have you travelled?”", d1_a_travel),
                link!("don't bother him with questions", d1_no_convo)
            );
        }
    }
}

#[ifview]
pub fn d1_t_city(s: &mut State) {
    s.relation_walker -= 1;
    p!("She smirks. “Which city? Why don't you guess?”");
    p!(
        "It's hard to say. Skin as pale as hers is uncommon in Hearth and Rye, but none of the city-states were ever composed of only one ethnicity. Without cultural tells, you can't hazard a guess. And she seems pleased to have stumped you."
    );
    p!(
        "Not long afterwards, she eases the vehicle to a stop. The sun is slanting low, and you'll need time to set up camp. The first day of travel is over. You're not sure quite what to expect from ",
        link!("the first night", first_night),
        "."
    );
}

#[ifview]
pub fn d1_t_driving(s: &mut State) {
    s.relation_walker += 1;
    p!(
        "She considers it for a moment. “New. Odd. But I'm not against it. It feels powerful. I'm used to relying on my own body for everything. It's a little warmer in here, though. Can you feel that?”"
    );
    p!(
        "It's true; the vehicle's engine-metabolism heats its metal body, and the cavity where you three sit is warm now, but the air comes in brittle wafts around the windshield. You know from experience, though, that adjusting to the cold will take a few days."
    );
    p!(
        "Not long afterwards, your saltwalker eases the vehicle to a stop. The sun is slanting low, and you'll need time to set up camp. The first day of travel is over. You're not sure quite what to expect from ",
        link!("the first night", first_night),
        "."
    );
}

#[ifview]
pub fn d1_t_travel(s: &mut State) {
    s.relation_walker += 1;
    s.used_1w_travel_ques = true;
    p!(
        "“Far east—all the way to Rye, but no further south. And a fair way north. I spend most of my time outside the cities; I like it better out here.” She gives a quick smile."
    );
    p!(
        "Not long afterwards, she eases the vehicle to a stop. The sun is slanting low, and you'll need time to set up camp. The first day of travel is over. You're not sure quite what to expect from ",
        link!("the first night", first_night),
        "."
    );
}

#[ifview]
pub fn d1_a_city(s: &mut State) {
    s.relation_walker += 2;
    s.used_1w_city_ques = true;
    p!(
        "He nods, his face crinkling into a smile. “My whole family; I can trace it back four generations. My grandfather's sibling was a high-ranking councillor. None of them were ever saltwalkers, before me. Bit of a break with tradition. I love the city well enough, but I couldn't imagine being confined to it my whole life. Or to any city at all.”"
    );
    p!(
        "He talks of the crispness of the salt wrack, the way its empty landscape makes him feel alive. Not long afterwards, he eases the vehicle to a stop. The sun is slanting low, and you'll need time to set up camp. The first day of travel is over. You're not sure quite what to expect from ",
        link!("the first night", first_night),
        "."
    );
}

#[ifview]
pub fn d1_a_driving(s: &mut State) {
    s.relation_walker += 1;
    p!(
        "“I don't hate it. It feels powerful. We're crossing ground much faster than I'm used to, and it'll spare our legs and backs the soreness. But I think it's easier to bear the cold when you're moving.”"
    );
    p!(
        "It's true; the vehicle's engine-metabolism heats its metal body, and the cavity where you three sit is warm now, but the air comes in brittle wafts around the windshield. You know from experience, though, that adjusting to the cold will take a few days."
    );
    p!(
        "Not long afterwards, your saltwalker eases the vehicle to a stop. The sun is slanting low, and you'll need time to set up camp. The first day of travel is over. You're not sure quite what to expect from ",
        link!("the first night", first_night),
        "."
    );
}

#[ifview]
pub fn d1_a_travel(s: &mut State) {
    s.relation_walker += 1;
    s.used_1w_travel_ques = true;
    p!(
        "“Mm. A little past Wick—that's near four hundred miles north of here, and further east than we'll be going, unless the Society was wrong about where the objective is. Didn't you do those triangulations, {s.addr}?” His voice is gruff, but there's the hint of a smile on his bearded face. “You'd best not be leading us astray. At any rate, I've gone further south than north. Down past Rye. You can still see where forests used to be, they're not all gone under the ice... but the salt is worse. More of it in the soil.”"
    );
    p!(
        "He tells you about the remnants of those trees: gray boughs like driftwood on land, petrified by minerals and the lack of decay. Not long afterwards, he eases the vehicle to a stop."
    );
    p!(
        "The sun is slanting low, and you'll need time to set up camp. The first day of travel is over. You're not sure quite what to expect from ",
        link!("the first night", first_night),
        "."
    );
}

#[ifview]
pub fn d1_no_convo(s: &mut State) {
    s.relation_walker -= 1;
    p!(
        "You don't speak with your companions, and they don't try to make conversation with you. Instead, you watch the barren glitter of the wrack as the sun shifts, until the saltwalker eases the vehicle to a stop. The sun is slanting low, and you'll need time to set up camp. The first day of travel is over. You're not sure quite what to expect from ",
        link!("the first night", first_night),
        "."
    );
}

#[ifview]
pub fn first_night(s: &mut State) {
    s.rations = s.rations.saturating_sub(3);
    p!(
        "As the ash-white land darkens in the dusk, the sky stays a clear pale blue for an hour or so. You work beneath it. It is wearying: to hammer stakes into the frozen ground, to pitch your thermal tent, running through the yet-unfamiliar checklist of your new routine. The saltwalker volunteers to cook dinner; this consists of boiling some sort of dumplings over a tiny stove. All your trash has to be discarded half a mile from the campsite. The saltwalker decrees this."
    );

    match s.walker {
        Walker::T => {
            p!(
                "She says it could attract things to the camp, otherwise. It's only a precaution, this close to a city, but you've seen the specimens and heard the stories. You don't disobey."
            );
        }
        _ => {
            p!(
                "He says it could attract things to the camp, otherwise. It's only a precaution, this close to a city, but you've seen the specimens and heard the stories. You don't disobey."
            );
        }
    }

    p!(
        "By the time it is well and truly night, the bleak strangeness of your situation is setting in. You feel empty, exposed, and very small for a short while. But your bed—well, the layers of coalsilk and oilfoam fluff which serve as one—is warm and soft. You cover your lamp with the overshirt you wore today, and close your eyes."
    );
    p!("What do you think about in the dark?");

    choice!(
        link!("the city of Hearth", think_hearth),
        link!("your purpose", think_purpose),
        link!("your misgivings", think_misgivings)
    );
}

#[ifview]
pub fn think_hearth(_s: &mut State) {
    p!(
        "Hearth is the home of scientific acclaim, and provides lavish support for its interpreters. It's why you chose to move there. The city-state is rich with resources, and maintains a polished, venerable air of respectability. A great oak tree grows outside Hearth's university, sheltered by a thin shining polymer structure that makes the campus temperate. The tree is around a hundred and fifty years old, if you recall correctly. The institution predates it."
    );
    p!(
        "But you can't help thinking of another city, the one you abandoned. As you fall into sleep, you wander its remembered streets. The high narrow canyons of steel and cement, cut into the overarching mountain; the trolleys, the blocks of apartments and workplaces. Massive bulks looming over shadowed gaps. Silhouettes loitering in loading bays and balconies."
    );
    p!(
        link!("When you wake,", day_two),
        " you don't recall any dreams. You almost never have them."
    );
}

#[ifview]
pub fn think_purpose(_s: &mut State) {
    p!("You reflect on the reason you're all here.");
    p!(
        "Your proposal for an expedition was beyond bold. Audacious, or foolhardy. Many interpreters before you have braved the salt wrack for specimens, maps, or other revelations of the landscape. But you, and your colleagues at the Observational Society, seek the center of the cataclysm that undid the world. To that end, you will travel north for hundreds of miles, through the desolate cold. What you are attempting has never been done. You will be legendary in science and history, if somehow you succeed."
    );
    p!(
        link!("When you wake,", day_two),
        " you don't recall any dreams. You almost never have them."
    );
}

#[ifview]
pub fn think_misgivings(s: &mut State) {
    s.dread += 1;
    p!(
        "Where did the Observational Society find your colleagues? You think of the brief meeting. Such a strange, formal ritual. It's almost as though you were prevented from talking to the others. Is there some secret purpose at work? Are you being set up for failure? You're sure that the Society wouldn't go that far, to the point of making an example of you. But maybe you and your colleagues are... disposable."
    );
    p!(
        "You don't know either of them. You're entrusting your life to them. Anyone who would willingly go out into the salt wrack has a certain disregard for safety, perhaps for sanity. That includes you, too."
    );
    p!(
        link!("When you wake,", day_two),
        " you don't recall any dreams. You almost never have them."
    );
}

#[ifview]
pub fn day_two(s: &mut State) {
    s.days = 2;
    s.miles += 17;
    s.north += 17;
    s.rations = s.rations.saturating_sub(3);

    p!(
        "The wind is stronger today, and your companions have a minor argument over the route you've mapped. Whether you're in the right place. “I know this valley,” the saltwalker says, gesturing freely. “Sometimes the weather troubles it. There's a shortcut; we usually make better time going this way.”"
    );
    p!("You check. No such route is marked on the map.");

    choice!(
        link!("follow the walker's suggestion", route_shortcut),
        link!("stay on the mapped route", route_mapped)
    );
}

#[ifview]
pub fn route_shortcut(s: &mut State) {
    s.miles += 42;
    s.north += 24;
    s.lateral += 18;

    p!(
        "The course takes you northeast, not strictly north, through a low-lying passage that might once have been a wide riverbed. But you do make good progress."
    );

    choice!(
        link!("volunteer to set up camp", camp_volunteer),
        link!("relax and let someone else do the work", camp_relax)
    );
}

#[ifview]
pub fn route_mapped(s: &mut State) {
    s.relation_walker -= 1;
    s.miles += 30;
    s.north += 30;

    match s.walker {
        Walker::A => {
            p!(
                "The saltwalker sighs, raising his eyebrows. He doesn't seem surprised, merely disappointed. “So you don't trust an old man's experience? Tell me again why I'm on this mission.”"
            );
        }
        Walker::T => {
            p!(
                "The saltwalker snorts. She doesn't seem surprised, merely disappointed. “Right. Don't bother saving yourself the trouble. We do it your way, {s.addr}.”"
            );
        }
        _ => {}
    }

    p!("You make fine progress, though not as much as you would have liked.");

    choice!(
        link!("volunteer to set up camp", camp_volunteer),
        link!("relax and let someone else do the work", camp_relax)
    );
}

#[ifview]
pub fn camp_volunteer(s: &mut State) {
    s.relation_walker += 2;
    s.relation_oracle += 2;
    s.rations = s.rations.saturating_sub(3);

    p!(
        "You earn grateful smiles from your colleagues. The work is exhausting, in a satisfying way. It warms you to the core, and you find yourself sweating under your thermal coat. When you get into your tent for the night, you have no trouble falling asleep, and no time to lie awake in thought. This ",
        link!("routine", day_three),
        ", it seems, is good for you."
    );
}

#[ifview]
pub fn camp_relax(s: &mut State) {
    s.dread -= 2;
    s.rations = s.rations.saturating_sub(3);
    s.used_backstory = true;

    p!(
        "It's a relief to be able to rely on your crew members. But once you're alone in your tent, you are unable to fall asleep easily. You lie awake as the tent ripples in the wind, and think about ",
        tun!("your home city", _backstory),
        ", for the first time in a long while."
    );
    p!(link!("And now you're out here.", day_three));
}

// ---------------- DAY 3 ----------------

#[ifview]
pub fn day_three(s: &mut State) {
    s.days = 3;
    s.miles += 67;
    s.north += 67;
    s.rations = s.rations.saturating_sub(3);

    p!(
        "With each passing day the strangeness recedes and you notice, more and more, the wrack's austere beauty, its complete sovereign self."
    );

    if s.north > 90 {
        EMBED!(preglacier_day_text);
    } else {
        EMBED!(aero_wreck);
    }

    EMBED!(bad_air);
}

#[ifview]
pub fn preglacier_day_text(_s: &mut State) {
    p!(
        "Yellow sky, gray land, sickly crystalline sun. You pass massive floes, heaped up as though this were the edge of an ocean."
    );
}

#[ifview]
pub fn aero_wreck(_s: &mut State) {
    p!(
        "Something is heaped up in the distance, standing out against the stark white field. Once you get close enough, you see that it's a mechanical carcass: the wreck of an aeromobile, a rare relic from Firmament's experimentation. Its metal shell is warped and buckled from the weight and movement of ice; it might have been downed decades ago. As far as you're aware, engineers have largely given up on the possibility of powered flight."
    );
    p!(
        "Odd scratches are raked into its body by the doors and windows. The saltwalker eyes them with suspicion as you pass by."
    );
}

#[ifview]
pub fn bad_air(s: &mut State) {
    p!(
        "While you start to set up camp, the walker is the first to spot the glyph, carved white into the side of a boulder. A horizontal line with a wavering vertical squiggle through the center of it."
    );
    p!("“That's the contamination sign.”");

    match s.walker {
        Walker::T => {
            p!(
                "She shakes her head, hunches her shoulders. “Something here is poisoned. That's what it says, {s.addr}. Could be you next, if you're bent on staying here. Maybe the ground's no good, maybe the ice. Sick water. Even if you distill it, might not clean it out.” She gives a slow, tense grin. “But I reckon I'll be all right no matter what. I've stayed in all sorts of wrong places.”"
            );
        }
        _ => {
            p!(
                "He squints doubtfully, looking around as if to spot signs of disease. “It's likely to be something in the ground. The soil. Before the saltfall, certain parts of the land were full of chemicals or worse. Even now, you stay away if you know where they are. It can seep up in the ice. Groundwater moves contamination.” He sighs. “Then again, it's hard to tell. We shouldn't put any ice here into the still, but if you want to make camp for a night, I think we'll have enough water anyways. I try not to be paranoid, but you always follow a saltsign if it's in your power.”"
            );
        }
    }

    choice!(
        link!("make camp under the sickwater sign", camp_sickwater),
        link!("travel by night", camp_travel_night)
    );
}

#[ifview]
pub fn camp_sickwater(s: &mut State) {
    s.rations = s.rations.saturating_sub(6);
    s.bad_air = true;

    p!(
        "You are careful. You use only the water you have in reserve canisters, more than enough to supply the three of you for a night. You try not to make contact with the ground."
    );
    if s.oracle != Oracle::V {
        p!(
            "In the middle of the night, you wake up coughing. It feels like you've inhaled smoke. But the feeling clears soon, and you sleep again."
        );
    }
    p!(
        "The morning sun is glassy, glaring through wisps of cloud that smudge into brilliant rainbows. A sun halo arcs across half of the sky."
    );
    if s.oracle == Oracle::V {
        p!(
            "The oracle looks a little peaky, as though they haven't slept well. But it's hard to tell with them sometimes. Truth be told, they usually look like that."
        );
    }
    p!(link!(
        "You head out into the brightness of day.",
        crate::saltwrack::chap2::day_four
    ));
}

#[ifview]
pub fn camp_travel_night(s: &mut State) {
    s.dread += 2;
    s.miles += 11;
    s.north += 6;
    s.lateral += 5;
    s.rations = s.rations.saturating_sub(6);

    p!(
        "The murky dusk deepens. A light snowfall starts to dance in the air, blowing sideways with the wind. Exhaustion makes you all achingly cold. Perhaps ten miles away from the site, your walker tells you it's safe to make camp. You eat a quick meal of packaged nutrition bars."
    );
    p!(
        "In the morning, your body feels stiff and slow to respond, still clinging to sleep. The sun glares down, splintering and smudging through your goggles, making the saltwrack glitter. A sun halo arcs across half of the sky, fringed by three points to the top and sides."
    );
    p!(link!(
        "You head out into the brightness of day.",
        crate::saltwrack::chap2::day_four
    ));
}
