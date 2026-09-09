use crate::saltwrack::{Oracle, State, Walker};
#[allow(unused_imports)]
use ifengine::elements::*;
use ifengine::ifview;

// ============================================================================
// CHAPTER 3: THE 400-MILE FRONTIER (Days 8 - 11)
// ============================================================================

// ---------------- DAY 8 ----------------

#[ifview]
pub fn day_eight(s: &mut State) {
    s.days = 8;
    s.rations = s.rations.saturating_sub(s.crew * 2);
    s.miles += 51;
    s.north += 51;

    if s.omen < 31 {
        s.miles = s.miles.saturating_sub(13);
        s.north = s.north.saturating_sub(13);
    }

    if s.lateral > 21 {
        EMBED!(waterfall_spring);
    } else {
        EMBED!(wrackplain_fly_day);
    }

    if s.fair_game {
        EMBED!(black_mold);
    } else {
        p!(
            "This is mapped terrain. One day soon, you'll be going where nobody else yet has, at least since the saltfall. But any given mile of the wrack has its surprises."
        );
        p!(link!(
            "Head onward toward the geothermal venting ahead.",
            hot_spring_yay
        ));
    }
}

#[ifview]
pub fn waterfall_spring(s: &mut State) {
    if s.omen < 31 {
        s.miles = s.miles.saturating_sub(3);
    }

    p!(
        "You pass through a landscape of short, gritty cliffs. Rectangular segments of rock lie littered in the snow beneath them. Lichens splotch the stone in unexpected colors: brilliant orange, soft green, scabby red."
    );
    p!(
        "Dark rivulets mark where water trickles from the cliffs. In some places little waterfalls have spurted out and frozen in long icicles. Then you come to a larger one, still flowing, that pools in a wide unfrozen spring. The water is flat and undisturbed. Only the smallest of ripples move in it."
    );

    match s.walker {
        Walker::T => {
            p!(
                "The saltwalker paces back and forth along a stretch of shoreline, looking for markers. An almost living heat pervades the metal of the vehicle, particularly in front, where the engine is housed."
            );
        }
        _ => {
            p!(
                "The saltwalker circles the shore, slowly and methodically, looking for signs. An almost living heat pervades the metal of the vehicle, particularly in front, where the engine is housed."
            );
        }
    }

    if s.omen > 82 {
        p!(
            "“Others have passed by this way before,” the saltwalker reports with a nod. “There's a sign: goodwater. It's safe. No contaminants. Let's fill our water supply here; you never know when you'll need it.”"
        );
    } else if s.omen > 30 {
        p!(
            "“There's no salt-sign here,” the saltwalker says. “We'll purify the water; it should be safe to use. It'll save us the trouble of melting ice, at least.”"
        );
    } else {
        p!(
            "“There's a sign,” the saltwalker warns grimly. “Others have passed by this way before. Sick land; something here is contaminated. I'm not sure what. It's best to leave places like this alone.”"
        );
        p!("You backtrack a little to find a suitable place to make camp.");
    }
}

#[ifview]
pub fn wrackplain_fly_day(_s: &mut State) {
    p!(
        "An uncharacteristic warmth brings little black flies to prick at your exposed skin. You wonder what they feed on, how long they spend frozen into the soil before a brief day of flight."
    );
}

#[ifview]
pub fn black_mold(_s: &mut State) {
    p!(
        "You heat up some canned food for dinner. When you open it, you're startled: a film of deep black fuzz floats atop the soup."
    );

    choice!(
        link!("throw out the rations", throw_out_rations),
        link!("view the mold under a microscope", view_mold_microscope),
        link!("eat the rations regardless", eat_rations_mold)
    );
}

#[ifview]
pub fn throw_out_rations(s: &mut State) {
    s.rations = s.rations.saturating_sub(6 + s.crew);
    s.dread = (s.dread - 1).max(0);

    p!(
        "This cannot be safe to consume. And now you have your doubts about the safety of the remaining supplies. You explain what you saw to the walker, hoping for approval, and you are vindicated."
    );

    match s.walker {
        Walker::A => {
            p!(
                "He looks dolefully down at the opened jar, and nods. “Where one is afflicted, others will surely be. Let's look through the rest of the rations, hm?”"
            );
        }
        _ => {
            p!(
                "“Seen this stuff before,” she says. “Good thing you brought this to me. We'll check if anything else is infected.”"
            );
        }
    }

    p!(
        "Together, you find a bag of rye flour and a portion of dried fruit which have been overgrown with the strange mold. Neither of you likes having to throw out food, but you feel better knowing that what you eat won't make you sick."
    );
    p!(
        "The walker is familiar with tomorrow's route, which promises to be ",
        link!("an easy enough voyage.", hot_spring_yay)
    );
}

#[ifview]
pub fn view_mold_microscope(s: &mut State) {
    s.dread += 1;
    s.rations = s.rations.saturating_sub(1 + s.crew);

    p!(
        "You scrape some of it onto a microscope slide and take it to your tent. Up close, there are none of the sporing bodies you expected. Instead, the mold is slick, black, angular, and jagged as the branches of coral. Is it mold at all? It is like a miniature city, you think. The microscope's lens creates chromatic aberration, making glassy rainbows play around the edges of the tiny structures. The word spire lodges in your mind, but you're not sure where you're recalling it from."
    );
    p!(
        "You write up your observations, considering that this is a kind of wrack biota you haven't seen before. You'll have to find something else for dinner."
    );
    p!(
        "The walker is familiar with tomorrow's route, which promises to be ",
        link!("an easy enough voyage.", hot_spring_yay)
    );
}

#[ifview]
pub fn eat_rations_mold(s: &mut State) {
    s.dread += 6;

    p!(
        "You are aware that this is a terrible idea in every sense. You are motivated not by a desire to preserve rations, but by manic curiosity. You skim off the patches of mold and thoroughly heat the contents of the jar, before taking a cautious spoonful. It tastes normal: watery broth made of vegetables and unspecified meat. Too bland. You finish it. You've almost forgotten about your decision by the time the toxins begin to affect you."
    );
    p!(
        "You find yourself twitching, shivering. Even through your thermal clothes, you feel cold and nauseous. Colors have become muted, as though you're seeing in the dark, in the grey and indistinct shades of night. But it's still early evening. The moon floats above the landscape, a huge white smear bordered by bright doubles of itself. A ring of light surrounds it like an eye. You lick your lips, run a hand over your face. A vast fear strikes you: the sun, the sun will eat you."
    );
    p!(
        "You stagger, fall, and lie down in the salty ice, unable to hold yourself up. Feverish tears stream down the sides of your face. Nobody can help you; you are more alone than you have ever been. And then you close your eyes, and see a pattern of swirling chaos that grips you entirely."
    );

    choice!(link!("succumb to delirium", succumb_delirium));
}

#[ifview]
pub fn succumb_delirium(s: &mut State) {
    s.days += 3;
    s.rations = s.rations.saturating_sub(s.crew * 6);

    p!(
        "You do not die, though at several points you want to. For days you suffer delirium, shaking and sweating in a nervous fever, plagued by unspeakable and relentless visions. Later you remember only fragments. A great warped space made of your own entrails—the constant sensation of falling—unable to communicate, to understand, gnawed at from under your skin, dislodged from the world, an awful music that never stopped. Monstrous faces in everything you saw. Worse when you closed your eyes."
    );
    p!("Once it's over, you are almost too weak to stand. Eyelike patterns linger in your vision.");

    let comp_desc = if s.crew >= 3 {
        "Your companions' relief is guarded, wary."
    } else {
        "Your companion's relief is guarded, wary."
    };
    p!("{comp_desc} There's no time to wait; your mistake has delayed the expedition.");
    p!(
        "Fortunately, the walker is familiar with tomorrow's route, which promises to be ",
        link!("an easy enough voyage.", hot_spring_yay)
    );
}

// ---------------- DAY 9 (THE GEOTHERMAL SPRING) ----------------

#[ifview]
pub fn hot_spring_yay(s: &mut State) {
    s.days += 1;
    s.rations = s.rations.saturating_sub(s.crew * 2);
    s.miles += 37;
    s.north += 37;

    p!(
        "The land is briefly volcanic. You follow the side of a fissure that snakes north-south. Refrozen meltwater has pooled in its depth. Beneath the snow, craggy dark grey stone glitters brittle and glassy. You see steam in the distance, and approach it; a hot spring smolders in a ring of mineral ice. Bubbles rise and hiss continually in the cloudy water."
    );

    choice!(
        link!("take a moment to relax here", relax_hot_spring),
        link!("keep heading north", keep_north_hot_spring)
    );
}

#[ifview]
pub fn relax_hot_spring(s: &mut State) {
    s.relation_oracle += 3;
    s.relation_walker += 3;
    s.dread = (s.dread - 3).max(0);

    p!(
        "The wind whips away the steam, but some of the water's heat still warms you if you crouch close."
    );

    if !s.walker.is_dead() {
        p!(
            "The walker has an idea, and pulls out packets of stimulant drink. You set your flasks in the shallow edges of the spring, and enjoy the hot beverage."
        );
    }

    if s.oracle == Oracle::S {
        p!(
            "The oracle pulls off their glove and tentatively splashes at the water. “It's warm,” they report. “Not too hot. Smells like sulfur, though.”"
        );
    }

    let group_desc = if s.crew >= 3 {
        "All of you crowd together, watching the water splutter. For a little while, your fears and doubts recede."
    } else {
        "The two of you crowd together, watching the water splutter. For a little while, your fears and doubts recede."
    };
    p!(group_desc);

    if s.days < 12 {
        p!(
            "You're making good progress so far—at least, ",
            link!("you hope so.", day_ten)
        );
    } else {
        p!(
            "You're behind the planned timeline. ",
            link!("You'll need to try to catch up.", day_ten)
        );
    }
}

#[ifview]
pub fn keep_north_hot_spring(s: &mut State) {
    s.miles += 24;
    s.north += 24;

    p!(
        "Fine snow begins to fall, muting out the world in blank soft white. There is nothing but the light prickling sound of snowflakes landing on your coat. Even the vehicle's heavy tread is almost silenced."
    );
    p!(
        "The saltwalker sets you to chiselling lumps of ice from the ground and running them through the purifier. You watch as they dissolve in the thermal module, leaving traces of sediment behind in the filter. Clean water trickles into the collecting jar."
    );

    if s.days < 12 {
        p!(
            "You're making good progress so far—at least, ",
            link!("you hope so.", day_ten)
        );
    } else {
        p!(
            "You're behind the planned timeline. ",
            link!("You'll need to try to catch up.", day_ten)
        );
    }
}

// ---------------- DAY 10 (THE 400-MILE BOUNDARY) ----------------

#[ifview]
pub fn day_ten(s: &mut State) {
    s.days += 1;
    s.rations = s.rations.saturating_sub(s.crew);

    p!(
        "You have journeyed beyond the known boundaries of Hearth's frontier, passing the four-hundred-mile threshold. The air is thinner here, sharp and metallic in your nostrils."
    );

    if s.north > 350 && s.north < 450 {
        EMBED!(big_lake);
    } else {
        EMBED!(motile_terrain);
    }
}

// Branch A: The Big Lake

#[ifview]
pub fn big_lake(_s: &mut State) {
    p!(
        "The mapped path takes you around the shoreline of a vast frozen lake, more like an inland sea. It would be quicker to simply cross over the ice; the vehicle's clawed feet won't slip. But you're not sure whether it's thick enough. You might be in danger of falling through."
    );

    choice!(
        link!("cross the lake", cross_the_lake),
        link!("take the longer route around", route_around_lake)
    );
}

#[ifview]
pub fn cross_the_lake(s: &mut State) {
    s.miles += 80;
    s.north += 80;
    s.rations = s.rations.saturating_sub(s.crew * 3);
    s.dread += 1;
    s.used_first_dream = true;

    p!(
        "You track a straight line north across the glassy white lake. The ice never fails. It must be a meter thick, at least; it doesn't crack once under the vehicle's weight."
    );
    p!(
        "Perhaps the lake enters your thoughts that night, because you dream for the first time in the wrack. A dream about the ocean. It rises around you, deep and hostile and wonderful. Though you have never felt the motion of waves, you know it now as you are caught in a swell of water, buoyed, churned down and sunken. Are you there at all? There seems to be nothing but empty dark brine, mountaining over itself as though it were all the planet contained. A sphere of depth, crested with sky."
    );
    p!(
        "The surface is far above you, awash with foreboding light. A pressure at your chest. Something below you."
    );
    p!(
        "It fragments and you ",
        link!("wake to an alien landscape.", spire_forest_day)
    );
}

#[ifview]
pub fn route_around_lake(s: &mut State) {
    s.miles += 80;
    s.north += 50;
    s.rations = s.rations.saturating_sub(s.crew * 2);
    s.dread += 1;
    s.used_first_dream = true;

    p!(
        "Rather than entrusting the vehicle to summer ice, you trace around the edges of the lake. You cover a lot of ground, but much of it is doubling back in a wide arc."
    );
    p!(
        "Perhaps the lake enters your thoughts that night, because you dream for the first time in the wrack. A dream about the ocean. It rises around you, deep and hostile and wonderful. Though you have never felt the motion of waves, you know it now as you are caught in a swell of water, buoyed, churned down and sunken. Are you there at all? There seems to be nothing but empty dark brine, mountaining over itself as though it were all the planet contained. A sphere of depth, crested with sky."
    );
    p!(
        "The surface is far above you, awash with foreboding light. A pressure at your chest. Something below you."
    );
    p!(
        "It fragments and you ",
        link!("wake to an alien landscape.", spire_forest_day)
    );
}

// Branch B: Motile Terrain

#[ifview]
pub fn motile_terrain(s: &mut State) {
    p!(
        "You are not where you should be. The topography of the map doesn't align with the landscape you find yourself in."
    );

    match s.walker {
        Walker::A => {
            p!(
                "The saltwalker indicates your surroundings, with unease in his voice. “Passed through here dozens of times. You see that boulder, like a monolith? Only thing is—it's about four hundred miles south of here, and much further east. I've heard of this happening, but never seen it myself 'til now. I'd say we just keep heading north until it sorts itself out.”"
            );
        }
        _ => {
            p!(
                "The saltwalker waves away your concerns. “It's not your calculations that're wrong. You've got us a few hundred miles away from where we are. Seen this sort of thing before. The terrain gets—misplaced.” She snorts resolutely. “We just keep heading north, it'll level out. Don't worry, {s.addr}.”"
            );
        }
    }

    if !s.oracle.is_dead() {
        p!(
            "The oracle shakes their head. “It feels like... hm.” They gesture loosely, hands twitching in the air as they try to map out some invisible shape. “If we go north from here, I can't promise it'll resolve like it should. We'll have to double back a bit. I can almost see where it ends.”"
        );
    }

    choice!(
        link!("double back to get out of it", motile_double_back),
        link!("head north regardless", motile_head_north)
    );
}

#[ifview]
pub fn motile_double_back(s: &mut State) {
    s.miles += 14;
    s.north += 7;
    s.lateral += 7;
    s.rations = s.rations.saturating_sub(s.crew);

    p!("After two hours or so, you make it safely out of the motile terrain.");

    if !s.oracle.is_dead() {
        p!(
            "The oracle guides you the whole time, as though they're seeing the shape of some invisible structure."
        );
        p!(
            "If the walker is the body and you are the mind, what does that make the oracle? An eye? A spirit? The role of an oracle is difficult to parse even without metaphor."
        );
    }

    p!(
        "Even once it clears, you don't quite trust the landscape anymore. You measure it in the corners of your vision, ",
        link!("trying to see anomalies.", day_eleven)
    );
}

#[ifview]
pub fn motile_head_north(s: &mut State) {
    s.miles += 15;
    s.north += 15;
    s.dread += 1;
    s.lateral = s.lateral.saturating_sub(39);
    s.rations = s.rations.saturating_sub(s.crew);

    p!(
        "The terrain begins to look right again after no more than two hours. There's a discrepancy, though, with your instruments. Your lateral reading is different; you've been displaced west by some distance."
    );

    if !s.oracle.is_dead() {
        p!(
            "If the walker is the body and you are the mind, what does that make the oracle? An eye? A spirit? The role of an oracle is difficult to parse even without metaphor."
        );
    }

    p!(
        "Even once it clears, you don't quite trust the landscape anymore. You measure it in the corners of your vision, ",
        link!("trying to see anomalies.", day_eleven)
    );
}

// ---------------- DAY 11 ----------------

#[ifview]
pub fn day_eleven(s: &mut State) {
    s.days += 1;
    s.rations = s.rations.saturating_sub(s.crew);

    if s.north > 350 && s.north < 450 {
        EMBED!(big_lake);
    } else {
        p!("You come across what you first think is a mirage. But no: it's a pond.");
        EMBED!(the_beach_that_kills_old_men);
    }
}

// Spire Forest (from Big Lake)

#[ifview]
pub fn spire_forest_day(s: &mut State) {
    s.miles += 57;
    s.north += 57;
    s.used_spire_forest = true;
    s.dread += 1;

    p!(
        "Your course takes you on a high ridge overlooking valleys flooded with brine, half-frozen saline sludge mirroring the cirrus-streaked sky."
    );
    p!(
        "You see something in the distance like an optical illusion. A series of bristling black towers, looking as though they burst upwards from the land. A conifer forest frozen in one stiff windless moment, or an abstract sculpture. Something about the forms is gnawing at your mind. Those thorny shapes, like pained coral."
    );

    if s.bad_air {
        p!("You remember, suddenly, how the oracle died.");
        choice!(link!("stay clear of them", avoid_spires));
    } else {
        choice!(
            link!("demand to stop here, to study the growths", study_spires),
            link!("stay clear of them", avoid_spires)
        );
    }
}

#[ifview]
pub fn study_spires(s: &mut State) {
    s.relation_walker -= 1;
    s.rations = s.rations.saturating_sub(7);
    s.dread += 3;

    if !s.walker.is_dead() {
        p!("Against the saltwalker's judgement, you approach the forest of negative-space towers.");
    } else {
        p!("You approach the forest of negative-space towers.");
    }

    p!(
        "They are made of flat hard stuff, carbon-black; you can fracture it with difficulty. No geometry, no pattern, is immediately evident in the branching forms, just an anarchy of shape."
    );
    p!(
        "You've gone far enough for today, and there's little time left. You make camp in an area well away from any of the bristling spires."
    );
    p!(
        "That night, the wind makes eerie fluting sounds as it rushes through the thorny maze. The spires are as deep black voids cut out against the brilliant star-filled sky."
    );
    p!(
        "You have no dreams, but you wake with a sense of unplaceable fear. For perhaps half a minute you are afraid to open your eyes. When you leave the tent, you see the spires in the morning light with an awful lurch. They seem too immediate, too concrete; they defy the salt wrack's desolate flat glaciality. There is something violent about them."
    );
    p!(
        "Some rations were ruined overnight. The packaging is split open. There are growths, perfect miniature versions of the spires, budding out. Without touching them, you toss the affected foods onto the ground."
    );
    p!(
        "When you leave, it is with the sense of escaping a trap. It's a relief when the forest is no longer visible in the distance."
    );
    p!(link!("Assess the expedition's status.", milestone_act1_end));
}

#[ifview]
pub fn avoid_spires(s: &mut State) {
    s.dread = (s.dread - 1).max(0);
    s.rations = s.rations.saturating_sub(s.crew);
    s.miles += 45;
    s.north += 34;
    s.lateral += 11;

    p!(
        "It is disappointing to pass up a research opportunity, but this is almost certainly for the better."
    );

    if !s.bad_air {
        p!(
            "You recall scattered mentions of some infectious black material, nightmare-tales of splinters that grew on their own."
        );
    } else {
        p!("You have learned too much about this spire-material already.");
    }

    p!("You make good progress for the rest of the day, leaving the grotesque silhouette behind.");
    p!(link!("Assess the expedition's status.", milestone_act1_end));
}

// The Beach / Mineral Pond (from Motile Terrain)

#[ifview]
pub fn the_beach_that_kills_old_men(s: &mut State) {
    s.miles += 68;
    s.north += 68;

    p!(
        "A rocky outcropping juts above the ice sheet like an island, and in its center is a shining mirror. It seems misplaced in the expanse of glacier. A relic from further south, from a subtly different climate. The land around it feels somehow blurry, warped at its edges."
    );
    p!(
        "You dismount from the vehicle to inspect it. The motionless water isn't frozen over, even though the shoreline is piled with chunks of ice. There are things in those glassy boulders: bubbles, dark fungal strips, fissures and boreholes. Is that a skeleton, thready white and delicate? It's too deep in to be sure."
    );
    p!(
        "The pond's ripples lap almost imperceptibly against a beach of dark silt, washed from the frosty soil. The water, you see as you come closer, is clear teal blue in its deepest center. A mineral color, devoid of life."
    );

    match s.walker {
        Walker::T => {
            p!(
                "You turn to the walker for guidance. She stares out at the little body of water. Her posture stiffens. “Doesn't seem right to me. Something about it...” She turns abruptly and paces around the shoreline. After a couple of minutes, she gives a short shout and points to something scratched into a rock."
            );
            p!(
                "It's a saltsign, the kind that walkers use to communicate about a location. You don't know this one: a loop encircling a dash, the ends closed with a slashing line."
            );
            p!(
                "“Means that this place is a trap. We should leave. I don't know what exactly is wrong here, but let's not wait to find out.”"
            );

            choice!(link!("keep heading north", avoid_pond));
        }
        Walker::A => {
            p!(
                "You turn to the walker for guidance. He stares out at the little body of water, brow furrowed under his hood. “I've never come across this one. It doesn't seem quite right to me, but I'm sure the water could be purified, if you want to stay here. Might be samples for you to take.”"
            );

            choice!(
                link!("stay by the pond", stay_by_pond),
                link!("keep heading north", avoid_pond)
            );
        }
        _ => {
            choice!(link!("keep heading north", avoid_pond));
        }
    }
}

#[ifview]
pub fn stay_by_pond(s: &mut State) {
    s.crew -= 1;
    s.walker = Walker::Dead;
    s.dread += 6;
    s.pool_death = true;
    s.rations = s.rations.saturating_sub(s.crew * 2);

    if !s.oracle.is_dead() {
        p!("The oracle wakes you up, urgently. Something is wrong.");
    }
    p!(
        "In the bleak cold light of morning, you see the pool of water. Something in it, meters from shore."
    );
    p!("It's the saltwalker. Facedown, floating. Motionless.");
    p!(
        "You think of going out to retrieve his body. But you'd have to come in contact with the water yourself. Wet clothing alone could be a death sentence out here, and you don't know what quality of the pond did this to him."
    );
    p!("You can't know.");

    if !s.oracle.is_dead() {
        p!(
            "The oracle is just as stricken as you are; they stare out at the water, twisting their gloves absently. “Is there nothing we can do? He was—” Their words fail. No, you are well aware: there is nothing to be done, now, other than trying to complete your expedition without a saltwalker to lead you."
        );
    } else {
        p!("And you're all alone now.");
    }

    p!(
        "Turning back is unthinkable. You've come this far already. You have to try to make it to the objective."
    );
    p!(link!("Assess the expedition's status.", milestone_act1_end));
}

#[ifview]
pub fn avoid_pond(s: &mut State) {
    s.miles += 12;
    s.north += 12;
    s.rations = s.rations.saturating_sub(s.crew);

    p!(
        "You make fine progress for the rest of the day, and sleep untroubled by dreams. Setting up camp is a little more arduous without a source of fresh water on hand, but you're used to using the purifier by now."
    );
    p!(link!("Assess the expedition's status.", milestone_act1_end));
}

// ---------------- MILESTONE COMPLETION (ACT 1: ~33%) ----------------

#[ifview]
pub fn milestone_act1_end(s: &mut State) {
    p!(s!("— MILESTONE REACHED: THE 400-MILE FRONTIER —")
        .cls("center")
        .style("font-size", "1.3rem")
        .style("font-weight", "bold")
        .style("margin-bottom", "1.5rem")
        .style("letter-spacing", "0.08em"));

    p!(
        "You stand at the threshold of the deep wrack. The mapped realm has been left behind; the coordinates on your charts now enter terra incognita. Behind you lie four hundred leagues of salt, frozen floes, and lost monuments. Ahead rise the sheer saw-tooth peaks of the northern range, where the great glacier pours down from the roof of the world."
    );

    p!("You have completed the first third (~33%) of the Saltwrack expedition.");

    let oracle_status = match s.oracle {
        Oracle::V => "Oracle (Vi) — Active",
        Oracle::S => "Oracle (S) — Active",
        Oracle::Dead => "Oracle — Deceased (Navigating by Journal)",
        Oracle::None => "Oracle — None",
    };

    let walker_status = match s.walker {
        Walker::A => "Saltwalker (A) — Active",
        Walker::T => "Saltwalker (T) — Active",
        Walker::Dead => "Saltwalker — Lost at the Mineral Pond",
        Walker::None => "Saltwalker — None",
    };

    p!(s!("Expedition Log:").style("font-weight", "bold"));
    p!("• Days elapsed: {s.days}");
    p!("• Total miles travelled: {s.miles} miles");
    p!("• Distance north: {s.north} miles");
    p!("• Remaining rations: {s.rations} rations");
    p!("• Psychological dread: {s.dread}");
    p!("• {oracle_status}");
    p!("• {walker_status}");

    p!(
        s!("To be continued in Act 2: The Northern Glacier & The Buried City.")
            .cls("center")
            .style("margin-top", "2rem")
            .style("font-style", "italic")
    );

    choice!(link!("Restart from Hearth", crate::saltwrack::chap1::p1));
}
