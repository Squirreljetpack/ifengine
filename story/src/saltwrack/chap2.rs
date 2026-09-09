use crate::saltwrack::{Oracle, State, Walker, chap1d::*};
#[allow(unused_imports)]
use ifengine::elements::*;
use ifengine::ifview;

// ============================================================================
// CHAPTER 2: THE DEEP WRACK (Days 4 - 7)
// ============================================================================

// ---------------- DAY 4 ----------------

#[ifview]
pub fn day_four(s: &mut State) {
    s.days = 4;
    s.rations = s.rations.saturating_sub(6);
    s.miles += 56;
    s.north += 56;

    if !s.bad_air {
        p!(
            "You're all still tired, and your pace is a bit slower today. But at least you're not making the journey on foot."
        );
    }

    if !s.used_vehicle_thought {
        p!(
            "For hours, all you hear is the hydraulic hiss and heavy crunching rhythm of ",
            tun!("the vehicle", _the_vehicle),
            "."
        );
    } else {
        p!(
            "For hours, all you hear is the hydraulic hiss and heavy crunching rhythm of the vehicle."
        );
    }

    p!(
        "The sky stays clear and empty; only the occasional wisp of cloud passes over. By the time the walker halts the machine, you are numb from the waist down, muscles cramped from sitting so long."
    );

    if s.oracle == Oracle::V {
        p!(
            "In the evening, while the walker finishes securing the anchors, you find yourself sitting near the oracle. ",
            link!(
                "They are reading by the light of a chemical lamp.",
                day_four_talk
            )
        );
    } else {
        p!(
            "You eat a sparse dinner in silence, gazing out into the twilight before crawling into your tent. ",
            link!("Sleep comes easily.", day_four_sleep)
        );
    }
}

#[ifview]
pub fn day_four_talk(s: &mut State) {
    s.relation_oracle += 1;
    p!(
        "The oracle glances up as you approach, then carefully marks their place in the thick notebook before closing it."
    );
    p!(
        "“The further north we go,” they say softly, their voice barely carrying over the wind, “the more the land resists being understood. In Hearth, people speak of the salt wrack as though it were merely empty. It isn't empty at all.”"
    );
    p!(
        "They offer you a cup of melted water warmed over their small heater. You sit together for a while, trading quiet observations of the day's journey, until the biting frost drives both of you to your sleeping rolls."
    );
    p!(link!("The night passes quietly.", day_five));
}

#[ifview]
pub fn day_four_sleep(s: &mut State) {
    if !s.used_backstory {
        p!(
            "You think of your former life in ",
            tun!("Firmament", _backstory),
            ", years ago."
        );
        p!(link!("And now you're out here.", day_five));
    } else {
        p!(
            "Despite the harsh conditions, you are ",
            link!("glad to be here", day_five),
            ", grateful for the brilliant blue sky and your own sense of purpose."
        );
    }
}

// ---------------- DAY 5 ----------------

#[ifview]
pub fn day_five(s: &mut State) {
    s.days = 5;
    s.miles += 23;
    s.north += 23;
    s.rations = s.rations.saturating_sub(s.crew);

    if s.lateral < 21 || s.bad_air {
        s.used_dead_walkers = true;
        EMBED!(dead_walkers_find);
    } else {
        s.used_scavenging = true;
        EMBED!(scavenging);
    }
}

#[ifview]
pub fn dead_walkers_find(_s: &mut State) {
    p!(
        "A mist hovers over the ice, shrouds the sun in steady unchanging grey. The land looks flat in such diffuse light."
    );
    p!(
        "You see the bright fabric of a tent, slumped under snow. And a body—two—lying nearby, similarly half-buried, in thermal clothing."
    );

    choice!(
        link!("investigate it", investigate_walkers),
        link!("don't bother with it", ignore_walkers)
    );
}

#[ifview]
pub fn investigate_walkers(s: &mut State) {
    s.dread += 2;
    s.miles += 11;
    s.north += 11;
    s.rations = s.rations.saturating_sub(s.crew);

    p!(
        "There were two travellers here. The snow has hardened into a crust over their bodies. As far as you can tell, there are no signs of a fight, or of anything else that might have killed them; it is as if they crawled out into the wrack and died, absent of purpose."
    );
    p!(
        "Perhaps you were hoping that any of their supplies might be useful to you, but it seems disrespectful to rob the dead, now that you're looking at their frozen faces. How long ago did this happen? Out here, bodies can linger strangely above ground for years, preserved by salt and subfreezing temperatures."
    );
    p!(link!("You're glad to leave the place behind.", day_six));
}

#[ifview]
pub fn ignore_walkers(s: &mut State) {
    s.miles += 32;
    s.north += 32;
    s.rations = s.rations.saturating_sub(s.crew);

    p!(
        "It is better, perhaps, to leave the dead to their icy graves. Out here, bodies can linger strangely above ground for years, preserved by salt and subfreezing temperatures. Nothing can be gained from prying around the corpses of strangers, save for a moment of macabre contemplation. And before long, you're far away, glad to ",
        link!("leave the place behind.", day_six)
    );
}

#[ifview]
pub fn scavenging(_s: &mut State) {
    p!(
        "It's not marked on your map, but in the distance you see the bulk of some building, tilted above the ice field. It looks lonely, half-buried, like a cinderblock dropped and embedded into the land."
    );
    p!(
        "“Might be something to scavenge in there,” the walker points out. “The structure looks stable.”"
    );

    choice!(
        link!(
            "“Let's see if there's anything of use.”",
            investigate_waystation
        ),
        link!("“We should keep moving.”", ignore_waystation)
    );
}

#[ifview]
pub fn investigate_waystation(s: &mut State) {
    p!(
        "You rummage through the structure; it seems like it used to be a waystation. It's almost entirely empty, rimed by salt stains and frost. There are a few canisters lying around, and rugged furniture."
    );

    match s.walker {
        Walker::A => {
            p!(
                "The walker rubs his chin. “No perishables, obviously. Not much for the taking. There's wood here. And an old-fashioned sort of fuel—our engine doesn't run on it, but it's highly combustible. We could have a real fire.”"
            );
        }
        _ => {
            p!(
                "The walker shrugs. “No food, obviously. Not much for the taking. There's wood here. And some old-fashioned fuel—our engine doesn't run on it, but it'll set things up to burn. We could have a real fire.”"
            );
        }
    }

    choice!(link!("make a campfire", make_campfire));
}

#[ifview]
pub fn make_campfire(s: &mut State) {
    s.dread = (s.dread - 4).max(0);
    s.relation_walker += 2;
    s.relation_oracle += 2;
    s.rations = s.rations.saturating_sub(s.crew);

    p!(
        "You help to strip chunks of salt-bleached wood from the frame of the structure. The saltwalker douses it in fuel and flicks a match to it, and you watch the hungry light kindle, reflected in the ice around it as it grows."
    );
    p!(
        "It brings you together, animates you. Someone makes a joke, or a terrible comment, it doesn't matter a moment later, but you're all laughing, and you feel a sudden fondness for your companions. The fire is wonderful, living, roving like an animal over the planks, a shade of orange you're not sure you've ever seen before. It ",
        link!("stays in your mind through the whole night.", day_six)
    );
}

#[ifview]
pub fn ignore_waystation(s: &mut State) {
    s.relation_walker -= 2;
    s.miles += 33;
    s.north += 33;
    s.rations = s.rations.saturating_sub(s.crew);

    match s.walker {
        Walker::A => {
            p!(
                "The walker nods, though he doesn't seem happy about it. He glances back to the desolate building until it's lost in the distance."
            );
        }
        _ => {
            p!(
                "The walker sighs. “Right. I don't like to pass up a chance to rummage around. But it's your mission.”"
            );
        }
    }

    p!(
        "Before long, you are ",
        link!("far", day_six),
        " from any signs of habitation, left once again to the empty white expanse."
    );
}

// ---------------- DAY 6 ----------------

#[ifview]
pub fn day_six(s: &mut State) {
    s.days = 6;
    s.rations = s.rations.saturating_sub(6);
    s.miles += 61;
    s.north += 60;
    s.lateral += 1;

    p!(
        "The saltwalker seems confident this morning, pointing at the map where a curve has been drawn in white ink. “This part's safer than most. The old route from Hearth to Clay passes through here. Nowadays, we use a faster path, but the ground here is quiet.”"
    );

    if !s.bad_air {
        p!(
            "The sky grows heavy and blue-purple, overburdened with moisture. As you stop to make camp for the night, it begins to snow. Thick clumps spiral out of the sky, landing with a puff against your sleeves and hood."
        );
        p!(
            "You eat dinner quickly, brushing snow out of your face every so often. The walker finds a pack of spiced biscuits in the rations and distributes them. As you lie in the quiet softness of your tent afterwards, appreciating the one time of day when you can take warmth for granted, you think about your colleagues."
        );
        p!("You reflect on...");

        choice!(
            link!("the oracle", reflect_oracle),
            link!("the walker", reflect_walker)
        );
    } else if s.oracle == Oracle::V {
        EMBED!(oracle_first_badair);
    } else {
        EMBED!(you_first_badair);
    }
}

// Day 6 - Reflections (Healthy path)

#[ifview]
pub fn reflect_oracle(s: &mut State) {
    match s.oracle {
        Oracle::V => {
            p!(
                "They are quiet and neurotically watchful, with a demeanor of wary stiffness. Physically smaller than you and the saltwalker, and fragile-looking. Perhaps sickly, perhaps just sleepless. They spend much of their free time scrawling down arcane notes in a thick book they carry."
            );
            if s.addr != "Interpreter" {
                p!("They unfailingly refer to you as “Interpreter”.");
            }
            p!("You find them to be...");

            choice!(
                link!(
                    "Unsettling. Less than an ideal colleague.",
                    oracle_v_unsettling
                ),
                link!("Polite. Nothing to complain about.", oracle_v_polite),
                link!("Helpful. You're glad they're around.", oracle_v_helpful)
            );
        }
        _ => {
            p!(
                "They are young, and seemingly inexperienced with the wrack, though they certainly have the oracular talent. Based on what they've told you, they see things that haven't yet come to pass, hidden ways and long-lost secrets. Visions and voices in dreams. None of this seems to disturb them."
            );
            p!("You find them to be...");

            choice!(
                link!("Strange. Less than an ideal colleague.", oracle_s_strange),
                link!("Harmless. Nothing to complain about.", oracle_s_harmless),
                link!("Sweet. You're glad they're around.", oracle_s_sweet)
            );
        }
    }
}

#[ifview]
pub fn oracle_v_unsettling(s: &mut State) {
    s.relation_oracle -= 2;
    s.rations = s.rations.saturating_sub(s.crew);
    p!(
        "You're not fond of them, of their ghostly presence in the crew. They keep to themself rather too much. Their freakishly pale eyes seem to look through you. Perhaps you'll get used to it. You need them, after all, or so the Observational Society was convinced. You'll make do with what you have."
    );
    p!(link!("Night deepens into bitter cold.", cold_snap));
}

#[ifview]
pub fn oracle_v_polite(s: &mut State) {
    s.relation_oracle += 1;
    s.rations = s.rations.saturating_sub(s.crew);
    p!(
        "You haven't been able to glean much personal information about them. They seem like a rational academic sort, unlike the few other oracles you've met. Your colleague is respectful, and they do their job well."
    );
    p!(link!("Night deepens into bitter cold.", cold_snap));
}

#[ifview]
pub fn oracle_v_helpful(s: &mut State) {
    s.relation_oracle += 3;
    s.rations = s.rations.saturating_sub(s.crew);
    p!(
        "They are endearing, in a twitchy sort of way. Earnest and grave, mostly, but you've seen them smile a handful of times. You weren't prepared for the depths of their understanding: so quick to make connections, to posit hypotheses. Their insights rival your own at times. You realize that you're genuinely fond of the oracle, grateful for their presence in your crew."
    );
    p!(link!("Night deepens into bitter cold.", cold_snap));
}

#[ifview]
pub fn oracle_s_strange(s: &mut State) {
    s.relation_walker -= 2;
    s.rations = s.rations.saturating_sub(s.crew);
    p!(
        "You doubt that your colleague has the experience they need to be useful. They seem unfamiliar with the wrack. They keep talking about their flights of fancy, and get lost in thought too easily. It makes you worry that they're detached from immediate reality. You need your crew to be aware and alert."
    );
    p!(link!("Night deepens into bitter cold.", cold_snap));
}

#[ifview]
pub fn oracle_s_harmless(s: &mut State) {
    s.relation_oracle += 1;
    s.rations = s.rations.saturating_sub(s.crew);
    p!(
        "You're not sure how much faith to put in the alleged touchsight and prophetic dreams of an oracle, but your colleague hasn't led you astray yet. And they're not one for arguing; in fact, they try hard to keep the atmosphere peaceful and optimistic."
    );
    p!(link!("Night deepens into bitter cold.", cold_snap));
}

#[ifview]
pub fn oracle_s_sweet(s: &mut State) {
    s.relation_oracle += 3;
    s.rations = s.rations.saturating_sub(s.crew);
    p!(
        "Their visions and dreams have been less useful, so far, than their atmosphere of meditative calm and optimism. They're sort of an anchor to your little group. Some people with extrasensory abilities and foresight can be shy, neurotic, or panicky; your colleague is none of these things. And more than that, they're kind. Compassionate. They seem to innately understand whenever you're not feeling right. You realize that you're genuinely fond of the oracle, grateful for their presence."
    );
    p!(link!("Night deepens into bitter cold.", cold_snap));
}

#[ifview]
pub fn reflect_walker(s: &mut State) {
    match s.walker {
        Walker::A => {
            p!(
                "He is self-assured, old enough to be a father to you and the oracle. He has a strong Hearth accent, and likes to tell tales of his travels. He's outgoing and friendly, though sometimes he scoffs at your ideas. Or anything you say that reminds him that you're from Firmament."
            );
            p!("You find him to be...");

            choice!(
                link!("Dreary. Less than an ideal colleague.", walker_a_dreary),
                link!("Prudent. Nothing to complain about.", walker_a_prudent),
                link!("Steadfast. You're glad he's around.", walker_a_steadfast)
            );
        }
        _ => {
            p!(
                "Your saltwalker is a bearish woman, just old enough to have a trustworthy working knowledge of the wrack. She's loud, and swift, and opinionated. At night, you hear her singing vulgar songs to herself in her tent. So far, you have been unable to determine where in the world she came from."
            );
            p!("You find her to be...");

            choice!(
                link!("Crude. Less than an ideal colleague.", walker_t_crude),
                link!("Driven. Nothing to complain about.", walker_t_driven),
                link!("Energizing. You're glad she's around.", walker_t_energizing)
            );
        }
    }
}

#[ifview]
pub fn walker_a_dreary(s: &mut State) {
    s.relation_walker -= 2;
    s.rations = s.rations.saturating_sub(s.crew);
    p!(
        "The walker is overly cautious, dismal, and unimaginative. He seems to disapprove of you in some vague way, either out of superstition or distaste for communalism. Most in Hearth still associate you with it, after all."
    );
    p!(
        "He's a stuffy old man who ought to have retired years ago. He gets in your way as often as he helps out. But you'll make do with what you have."
    );
    p!(link!("Night deepens into bitter cold.", cold_snap));
}

#[ifview]
pub fn walker_a_prudent(s: &mut State) {
    s.relation_walker += 1;
    s.rations = s.rations.saturating_sub(s.crew);
    p!(
        "He's an undeniably trustworthy guide, given his decades of experience. He seems about the standard for saltwalkers: an old sel with a head full of improbable, hair-raising stories. He's keen, cautious, and professional."
    );
    p!(link!("Night deepens into bitter cold.", cold_snap));
}

#[ifview]
pub fn walker_a_steadfast(s: &mut State) {
    s.relation_walker += 3;
    s.rations = s.rations.saturating_sub(s.crew);
    p!(
        "He has a paternal air about him, though you're certain he has no children. He's a veteran both at traversing the salt wrack and at the subtler art of lending support to a group. You trust him innately. You realize you're genuinely fond of the walker, grateful for his presence."
    );
    p!(link!("Night deepens into bitter cold.", cold_snap));
}

#[ifview]
pub fn walker_t_crude(s: &mut State) {
    s.relation_walker -= 2;
    s.rations = s.rations.saturating_sub(s.crew);
    p!(
        "She's unprofessional and irreverent; perhaps younger than you, but delights in ordering you around. A careless, unsophisticated bully. Her personal habits are irritating. And that visor. You can't avoid the subject: it makes her face inhuman, unreadable. At times, she frightens you."
    );
    p!(link!("Night deepens into bitter cold.", cold_snap));
}

#[ifview]
pub fn walker_t_driven(s: &mut State) {
    s.relation_walker += 1;
    s.rations = s.rations.saturating_sub(s.crew);
    p!(
        "You don't know much about saltwalker culture. From what you've heard, her behavior doesn't stray too far from expectations. Her ceaseless energy is a boon; she'll pick up any task you foist off on her."
    );
    p!(link!("Night deepens into bitter cold.", cold_snap));
}

#[ifview]
pub fn walker_t_energizing(s: &mut State) {
    s.relation_walker += 3;
    s.rations = s.rations.saturating_sub(s.crew);
    p!(
        "You have a tendency to get lost in your thoughts, but it's hard to be melancholy when she's around. Her relentless attitude is infectious; her strength, optimism, and courage uplift you. You realize you're genuinely fond of the walker, grateful for her presence in your crew."
    );
    p!(link!("Night deepens into bitter cold.", cold_snap));
}

// Day 6 - Sickness Branch (Oracle V path)

#[ifview]
pub fn oracle_first_badair(_s: &mut State) {
    p!(
        "The oracle has a persistent cough that develops quickly over the day, until they are doubled over in their seat of the vehicle, gasping for breath. They insist that they're fine; they insist that you keep going."
    );
    p!(
        "At the end of the day, they wince every time they cough. When you ask, they mutter that it feels like something's clawing at their lungs."
    );
    p!(
        "The saltwalker watches them apprehensively. “I don't like that. We have antibiotics; I'll make sure they get some.”"
    );
    p!(link!(
        "The night passes, and you do not dream.",
        night_passes_no_dream
    ));
}

#[ifview]
pub fn night_passes_no_dream(s: &mut State) {
    s.days += 1;
    s.rations = s.rations.saturating_sub(s.crew);
    s.miles += 6;
    s.north += 6;

    p!("The sun rises. Extraordinary prisms of rime glint on the ground.");
    p!(
        "The antibiotics have done nothing for your oracle. Their breathing is labored, shuddery. But their face is grimly set as they board the vehicle, insisting they'll be fine. In the cold air, their condition worsens rapidly, though they try to hide it. They gaze out at the wrack and occasionally flinch, startled, at something you can't see. At one of these points they gasp and slump forward, wrenched by a violent spasm."
    );
    p!(
        "The sleeve of their coat is spattered with vibrant blood. It's dripping from their mouth. They turn to you, eyes wide with true fear."
    );
    p!(
        "You stop immediately. When the three of you ",
        link!("dismount", dismount_oracle),
        " from the vehicle, the saltwalker has to carry the oracle down."
    );
}

#[ifview]
pub fn dismount_oracle(s: &mut State) {
    s.days += 3;
    s.rations = s.rations.saturating_sub(18);

    p!(
        "You set up camp as fast as possible, in as wind-sheltered a place as can be managed. And for several days you wait, as your oracle wastes away."
    );
    p!(
        "You and the saltwalker check on them constantly, but all your curative efforts come to nothing. The disease runs its swift and inevitable course. Your companion remains lucid; every time you see them, they're methodically making notes in the journal they carry. Finally you come to their tent just to ask what they're working on."
    );
    p!(
        "They struggle to sit up, strands of unkempt hair hanging in their face. You smell the close sick scent of their sweat. “I had wanted so badly to see the objective. You have no idea how much... But, Interpreter, we both know I'm not going to make it much longer.”"
    );
    p!(
        "Their face is gaunt and anemic, skull-hollows visible under their skin. They tap the notebook lying next to their hand. “I'm writing out everything I can see ahead. Hazards, strange parts of the land... even dreams. I don't know, I'm trying to be rigorous about it, I'd hate to lead you astray, but—” They break off for a moment, struggling to breathe. “I need to contribute as much as I can. I was meant to be here. I'd rather die here than live anywhere else.” They run a hand fretfully through their hair. “Please. If you can, even without me... do your best to get there. Someone ought to know.”"
    );

    choice!(
        link!(
            "“We'll see. Without an oracle, that may be impossible.”",
            oracle_dying_pessimistic
        ),
        link!("“I promise we'll try.”", oracle_dying_promise),
        link!(
            "“I promise we'll try, Interpreter.”",
            oracle_dying_promise_addr
        )
    );
}

#[ifview]
pub fn oracle_dying_pessimistic(_s: &mut State) {
    p!(
        "“I know... I wish you would try. You've made it possible for us to be here. For me to...” You watch them realize, belatedly, what they've said. “I mean—no, I wasn't blaming you, I mean that I'd rather be here, no matter what. I hope I can still be of service to your expedition.”"
    );
    p!(
        "Another coughing fit catches them. They curl over, torn by air that grates and rasps in their chest, clutching their notebook as though the weight of their work steadies them."
    );
    p!(
        "You find it almost impossible to think of creating a rigid science from oracular sense: methodical analysis of dreams and touchsight portents, rendered into instruction, legible and rational. It seems like a contradiction. But they're managing it. Or at least so you hope. ",
        link!("You have to hope.", oracle_rip_morning)
    );
}

#[ifview]
pub fn oracle_dying_promise(s: &mut State) {
    s.relation_oracle += 2;
    p!(
        "“Thank you, Interpreter.” Their hand wanders over to grip your wrist lightly, absently. “I wish—oh, but there's no use in that now.” They give a wry fleeting smile that wavers and disappears as they're caught by another coughing fit. They curl over, torn by air that grates and rasps in their chest, clutching their notebook as though the weight of their work steadies them."
    );
    p!(
        "You find it almost impossible to think of creating a rigid science from oracular sense: methodical analysis of dreams and touchsight portents, rendered into instruction, legible and rational. It seems like a contradiction. But they're managing it. Or at least so you hope. ",
        link!("You have to hope.", oracle_rip_morning)
    );
}

#[ifview]
pub fn oracle_dying_promise_addr(s: &mut State) {
    s.relation_oracle += 4;
    p!(
        "The oracle's eyes go wide and their mouth opens a little—silent, stricken, almost pathetic in their gratitude before they regain some composure, wincing back tears. “Thank you. Thank you...” Their hand wanders over to grip your wrist lightly, absently. “I wish—oh, but there's no use in that now.” They give a wry fleeting smile that wavers and disappears as they're caught by another coughing fit. They curl over, torn by air that grates and rasps in their chest, and clutch their notebook as though the weight of their work steadies them."
    );
    p!(
        "You find it almost impossible to think of creating a rigid science from oracular sense: methodical analysis of dreams and touchsight portents, rendered into instruction, legible and rational. It seems like a contradiction. But they're managing it. Or at least so you hope. ",
        link!("You have to hope.", oracle_rip_morning)
    );
}

#[ifview]
pub fn oracle_rip_morning(s: &mut State) {
    s.days += 1;
    s.rations = s.rations.saturating_sub(9);

    p!(
        "The next morning the saltwalker stops to talk to you. You fear, for a second, that the oracle is dead."
    );

    match s.walker {
        Walker::T => {
            p!(
                "Not yet, though. “I've given them a sedative. For the pain.” She lowers her head. You imagine she's looking into your eyes, behind that strange half-helm. “Seems like they're going to die today. Can't be sure, but I can't imagine they'll last much longer.”"
            );
        }
        _ => {
            p!(
                "Not yet, though. “I've given them a sedative. For the pain.” He looks askance, the creases around his eyes deepening as his brow furrows. “Seems like they're going to die today. Can't be sure, but I can't imagine they'll last much longer.”"
            );
        }
    }

    p!(
        "You wait around camp. You could be writing up your findings, or examining samples, but you can't focus on work right now. Twice you go into the oracle's tent to check on them, but they're either asleep or too far gone to respond to you. After sunset, you leave them alone, so as not to let in the brutally cold night air."
    );
    p!("You haven't managed to sleep yet when the saltwalker retrieves you.");

    let p_desc = match s.walker {
        Walker::A => "His face is an eerie statue in the chemical lamp's white light. He",
        _ => "Her face is an eerie statue in the chemical lamp's white light. She",
    };
    p!("{p_desc} takes your arm and leads you to your dying companion.");
    p!(link!("Go into the tent.", oracle_passes_away));
}

#[ifview]
pub fn oracle_passes_away(s: &mut State) {
    s.dread += 6;
    s.crew -= 1;
    s.oracle = Oracle::Dead;

    p!(
        "The oracle lies in a heap of coalsilk blankets. They move weakly, shuddering and turning their head, spurred by some uncertain impulse. Their eyes are glassy and unfocused; you recall the walker's explanation about sedatives. Even so, you glimpse a dim terror there, and wonder what they can see now that you cannot."
    );
    p!(
        "They prop themself up to retch over the floor of the tent. Frothy blood flows from their mouth, and a handful of black spiky fragments. The oracle falls onto their side, convulsing, twitching over and over, as blood rattles in their throat. Less now. Their movements weaken and slow. A final shiver. You see their eyes change, the icy irises eclipsed by pupils that slide wide and empty."
    );

    match s.walker {
        Walker::T => {
            p!(
                "The walker presses her fingers to their wrist for a moment and nods confirmation of the death."
            );
        }
        _ => {
            p!(
                "The walker presses his fingers to their wrist for a moment and nods confirmation of the death."
            );
        }
    }

    p!(
        "Throughout it all, you sat still: immobilized by horror, by your own regret and misery. You had no words of comfort to offer. There was nothing more you could have done. Now, at least, it is over."
    );
    p!(
        "You reach for one of the black barbs, to pick it up and examine it. The walker grabs your hand. “Don't touch those.”"
    );

    choice!(
        link!(
            "observe the saltwalker funerary rites",
            observe_funerary_rites
        ),
        link!("ask to perform an autopsy", perform_autopsy)
    );
}

#[ifview]
pub fn observe_funerary_rites(s: &mut State) {
    match s.walker {
        Walker::T => {
            p!(
                "The walker leaves for a time, and comes back holding a stone with a symbol scratched into it. A circle tangent to a horizontal line on one side, with a vertical line protruding from the other. The salt-sign of death; a traditional grave marker, out here."
            );
            p!(
                "She places it at the oracle's head. All her usual energy is absent; she just looks defeated. She says nothing."
            );
            p!(
                "Later, she places a hand on your shoulder. “You're not going to stop, are you? Good. They wanted you to get to your objective. I... I don't care, myself, but I'll do my best to bring you there safely.”"
            );
            p!(link!(
                "Prepare to navigate by their journal.",
                journal_navigation
            ));
        }
        _ => {
            p!(
                "The walker leaves and comes back with a little piece of wire, twisted into a symbol. A circle tangent to a horizontal line on one side, with a vertical line protruding from the other. The salt-sign of death: a traditional grave marker, out here."
            );
            p!(
                "He places it in the oracle's hand and pulls the covers back up around their body, as though leaving them to sleep. You hear him murmur: “You deserved better than this.”"
            );
            p!("Afterwards, he meets you outside. “{s.addr}, I think we should turn back.”");

            choice!(
                link!("hear the walker out", hear_walker_out),
                link!("refuse to consider it", refuse_turn_back)
            );
        }
    }
}

#[ifview]
pub fn perform_autopsy(s: &mut State) {
    s.relation_walker -= 5;
    s.dread += 3;

    match s.walker {
        Walker::T => {
            p!(
                "Her lip curls. “Truly? You have to be doing this? You've seen what you've seen. Isn't that enough? You're not going to find out anything else, Interpreter. If you want to, I won't stop you, but I won't help to disturb the dead either.”"
            );
        }
        _ => {
            p!(
                "He looks at you with a grave expression. With suspicion, you realize. “Isn't that... disrespectful? I wouldn't treat your body that way, if you died, forbid the thought. Prying someone open. I don't know what kind of mourning you're used to, but I don't like that. Do it, if you must. If you think it'll help somehow.”"
            );
        }
    }

    p!(
        "The tent wouldn't have been used again anyway; you decide to operate inside it, rather than in the freezing cold. There is a small saw provided with your dissection supplies. It proves difficult to use on bone. But you manage. When you pry open the oracle's ribcage, you're not prepared for the sight within."
    );
    p!(
        "The lung tissue is torn to shreds, clotted with that same thorny black growth. Little spiky clusters have spread throughout their thoracic cavity, adhering to diaphragm and intercostal muscles, punching into the esophagus. It is not entirely like any organism you have seen before. The ecology of the wrack is full of brutal surprises."
    );
    p!(
        "But you know this: it grew in their lungs first and foremost. It spread through the air. The poisoned air, in that place where you chose to stay. This is your fault."
    );
    p!(
        "You thought you took every precaution; you are certain that you did not come into contact with the sharp growths. But the oracle's blood gets under your gloves. Under your fingernails. Your hands stink of iron for days after."
    );
    p!(link!(
        "Prepare to navigate by their journal.",
        journal_navigation
    ));
}

#[ifview]
pub fn hear_walker_out(s: &mut State) {
    s.relation_walker += 2;
    p!(
        "You listen, and try to be fair. But you've worked toward this expedition for years. This might be the most significant thing you do in your life. Turning back is unthinkable. And the oracle asked you to see the objective point. After their sacrifice, it seems all the more necessary."
    );
    p!(
        "You explain all this to the saltwalker. He pleads with you, again. But you are adamant. Righteous determination fills you to the core: you will go north, and you will witness what nobody else yet has."
    );
    p!(
        "Eventually, he nods, defeated. “If I can't stop you, the best I can do is to make sure you get there safely.” He glances back to the tent in which your colleague's body lies. “It's late. We need to sleep. We'll keep travelling in the morning.”"
    );
    p!(link!(
        "Prepare to navigate by their journal.",
        journal_navigation
    ));
}

#[ifview]
pub fn refuse_turn_back(s: &mut State) {
    s.relation_walker -= 1;
    p!(
        "You've worked toward this expedition for years. This might be the most significant thing you do in your life. Turning back is unthinkable. And the oracle asked you to see the objective point. After their sacrifice, it seems all the more necessary."
    );
    p!(
        "You explain all this to the saltwalker. He pleads with you, again. But you are adamant. Righteous determination fills you to the core: you will go north, and you will witness what nobody else yet has."
    );
    p!(
        "Eventually, he nods, defeated. “If I can't stop you, the best I can do is to make sure you get there safely.” He glances back to the tent in which your colleague's body lies. “It's late. We need to sleep. We'll keep travelling in the morning.”"
    );
    p!(link!(
        "Prepare to navigate by their journal.",
        journal_navigation
    ));
}

#[ifview]
pub fn journal_navigation(s: &mut State) {
    s.days += 1;
    s.journal_nav = true;
    s.rations = s.rations.saturating_sub(s.crew * 2);

    p!(
        "The loss is devastating in a metaphysical way. Without your oracle, you are blinded. The uncanny sixth sense that some call “touchsight” can navigate the wrack more reliably than a compass. But you and the saltwalker must rely on what is visible."
    );
    p!("The rations you have will last you longer, now. But that's cold comfort.");
    p!(
        "The oracle mapped out a path for you to follow. They describe particular features of the land to orient you, should you stray from this route. There are two passes through the mountains, onto the vast northern glacier. They instructed you to take the further one, to the east, explaining that it is less risky. The glacier itself is described in painstaking detail, often down to the mile—despite the fact that nobody has ventured so far north."
    );
    p!(
        "Their journal is not entirely given over to these instructions. There are side notes about dreams or premonitions that you cannot parse. There are sketches: anatomies, wrack biota, things that look like stars or solar flares. Questions that can never now be answered."
    );
    p!(
        "With the journal, and the saltwalker's guidance, you prepare to ",
        link!("head out once more.", crate::saltwrack::chap3::day_eight)
    );
}

// Day 6 - Sickness Branch (Player catches spirelung - Bad End)

#[ifview]
pub fn you_first_badair(_s: &mut State) {
    p!("It is a good day, by anyone's standards.");
    p!(
        "But you feel a scratching in your throat, or deeper down in your chest. A migraine looms, pulses, threatens to burst upon you. It seems too trivial to complain to your colleagues about, but it worries you nevertheless."
    );
    p!(
        "The corner of your eye itches. You rub at it. Something like black sand comes away on your fingertip."
    );
    p!(
        "If you're still feeling ill tomorrow, you reason, ",
        link!("you can tell them then.", you_ouch)
    );
}

#[ifview]
pub fn you_ouch(s: &mut State) {
    s.days += 1;
    s.rations = s.rations.saturating_sub(s.crew * 2);

    p!(
        "It's a bad night. You thrash in your sleep, sweating through your bedroll, and wake with a fever. The left side of your vision is overtaken by crawling forms that flit and jerk when you move your head. The antipyretics in your medical supply don't help much; you have a hopeless feeling of slipping from lucidity. Your chest is tight."
    );
    p!(
        "You say something to the oracle, and realize you're not making any sense. Something hot and wet in your throat, and a visceral flood of metal-taste. Grainy black particles in your spit, fragments as sharp and brittle as glass."
    );
    p!("There's something hard in your ear,");
    p!(link!("the rest is only half-there...", you_spirelung));
}

#[ifview]
pub fn you_spirelung(s: &mut State) {
    s.days += 4;
    p!(
        "You are terribly ill already; perhaps it is fortunate that, in your delirium, you can make little sense of things. The flow of one event to another is staccato, interrupted, seen only in warped moments. Something deeper hums within you now, drawing you away. You finally understand—either in the diseased mire of your mind, or in overheard pieces of your colleagues' conversations. You must have inhaled an infectious thing, in that place where you stayed. Some sort of particles, a microbe, a parasite. But by now you are awash in hallucinations, and your companions can do nothing for you. It bores out through the soft parts of your head, insistent as a fungus. It jags up and out through your left eye. Even then you do not die, not yet. You are too good a host."
    );
    p!(
        "It is the saltwalker who finally insists on leaving you. By then you are unrecognizable. You lie alone in a tent, bristling with fervent growths like black coral. It consumes you and grows strong and tall and ripe. It prepares to release spores."
    );
    p!("Eventually the cold claims you. With so little of you left, it's not much of a mercy.");

    p!(s!("— GAME OVER: CLAIMED BY SPIRELUNG —")
        .cls("center")
        .style("margin-top", "2rem")
        .style("color", "#c33"));
    choice!(link!("Restart Expedition", crate::saltwrack::chap1::p1));
}

// ---------------- DAY 7 (COLD SNAP) ----------------

#[ifview]
pub fn cold_snap(s: &mut State) {
    p!(
        "Aside from getting to know your colleagues, the expedition is going well enough, so far. The terrain is familiar, and you're travelling on well-mapped land. You are beginning to think of the basic conditions of the wrack as constant, holding no or little surprise."
    );
    p!("You are wrong.");
    p!(
        "Overnight an impossible cold moves into the area. You feel it in your bedroll, when you wake briefly to shiver and curl up beneath layers and layers of insulating fluff. In the morning a thick and angry band of clouds blots out the sky, as wind scours the barren plain. The temperature measures 30 degrees below freezing. After breakfast, it is 35. Nobody wants to take the tents down."
    );

    p!(
        "You ask the saltwalker what to do. {s.walker.p()} shrugs. “It's a storm vortex. We hunker down. We wait it out. Unless you want to lose your toes and fingertips.”"
    );

    choice!(
        link!("stay put", stay_put),
        link!("the idea of waiting is unacceptable", force_march)
    );
}

#[ifview]
pub fn stay_put(s: &mut State) {
    s.days += 1;
    s.rations = s.rations.saturating_sub(s.crew);

    p!(
        "There's no point in risking injury. You're well-stocked with supplies; you can wait out this cold front, however long it takes."
    );
    p!("You'll have a lot of time to kill.");

    choice!(
        link!("talk to the oracle", oracle_convo_coldsnap),
        link!("talk to the walker", walker_convo_coldsnap),
        link!("relax on your own", relax_on_your_own),
        link!("keep up your studies", keep_up_studies)
    );
}

#[ifview]
pub fn force_march(s: &mut State) {
    s.days += 1;
    s.rations = s.rations.saturating_sub(s.crew);

    p!(
        "How long will this last? You don't know. It could get colder; it could be 50 below by tomorrow. You can't afford to lose any more time than necessary."
    );
    p!(
        "“If we head east,” the saltwalker points out, “we'll have an easier time, based on the angle of the wind.”"
    );

    choice!(
        link!("align yourself north", align_north),
        link!("align yourself east", align_east)
    );
}

#[ifview]
pub fn align_north(s: &mut State) {
    s.north += 15;
    s.miles += 15;
    s.rations = s.rations.saturating_sub(s.crew);

    p!(
        "You don't want to have to double back later. Heading east might result in lost time anyways."
    );
    p!(
        "The scalding cold wind, nearly opaque with tiny shards of ice, isn't easy to bear no matter which direction you're facing. Any exposed skin goes livid and numb in minutes. You are forced to travel more slowly, and after a mere few hours none of you can take it anymore."
    );
    p!(
        "But the cold snap abates ",
        link!("the next morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn align_east(s: &mut State) {
    s.lateral += 20;
    s.north += 4;
    s.miles += 24;
    s.rations = s.rations.saturating_sub(s.crew);

    p!(
        "The scalding cold wind, nearly opaque with tiny shards of ice, isn't easy to bear no matter which direction you're facing. Any exposed skin goes livid and numb in minutes. You are forced to travel more slowly, and after a mere few hours none of you can take it anymore."
    );
    p!(
        "But the cold snap abates ",
        link!("the next morning.", crate::saltwrack::chap3::day_eight)
    );
}

// Cold Snap Dialogues & Activities

#[ifview]
pub fn oracle_convo_coldsnap(s: &mut State) {
    match s.oracle {
        Oracle::V => {
            s.used_wick = true;
            p!(
                "Your colleague is reclusive, somewhat of a mystery. It might help if you could get to know them better."
            );
            p!(
                "There's a question you've been meaning to ask. Of little consequence, really, out here. Still. Still you want to know."
            );
            choice!(link!("“Are you from Firmament too?”", vi_wick_convo));
        }
        _ => {
            s.used_art = true;
            p!(
                "Your colleague is still somewhat of a mystery. It might help if you could get to know them better."
            );
            choice!(
                link!("“Which city did you come from?”", s_city_convo),
                link!("just let them talk", s_exile_talk)
            );
        }
    }
}

#[ifview]
pub fn vi_wick_convo(_s: &mut State) {
    p!("The oracle adjusts their spectacles. “Ah. No. I grew up in Wick.”");
    p!(
        "Oh. That makes some sort of sense. Wick is a tiny city-state to the north of Firmament, founded by restive self-imposed exiles from the cooperative kamis state. Those who prioritized their individuality over the collective good, hundreds of years ago. Profiteers, or those who wished to be. Today, Wick is ideologically and economically unimposing. It gets dark, so you've heard, in winters."
    );
    choice!(link!("“What was it like there?”", vi_wick_story));
}

#[ifview]
pub fn vi_wick_story(_s: &mut State) {
    p!(
        "The oracle's expressions are often hard to read. As they begin, their voice is soft and almost monotone."
    );
    p!(
        "“There were sparse conifer forests, in the reclaimed soil. Public gardens of a sort. Never so grand as the oaks and sycamores in Hearth. But I—I liked that, when I was young. It was quiet there.”"
    );
    p!(
        "“In Wick, there's... disparity. It's a small place, without many resources. My family was fortunate... we weren't laborers. We had more than what we needed. But it wasn't—it's not—a functioning system. Not really. It breaks down.”"
    );
    p!(
        "“There was an antibiotic shortage, for a year or two. I don't know whether the issue was manufacturing or distribution... I was eleven years old. My little brother was three.” They blink hard. “He developed bacterial meningitis. He died.”"
    );
    p!(
        "The oracle looks up into your eyes for just a moment. “That. That wouldn't have happened in Firmament, would it?”"
    );

    choice!(
        link!("“No. Never. I'm—I'm sorry.”", vi_firmament_sorry),
        link!(
            "“Not that. But I've seen children die in other ways.”",
            vi_firmament_other
        )
    );
}

#[ifview]
pub fn vi_firmament_sorry(_s: &mut State) {
    p!(
        "“Mmh. It was a long time ago. But I couldn't stay in Wick. And...” They tilt their head to the side. “You went to Hearth too. I won't speculate. Hearth is abundant in resources, especially for academics... I was a research assistant, for a long time. I liked it. At any rate, Hearth wouldn't be nearly so grand without the materials mined by Clay. It makes me feel strange, living off someone else's labor. I've never seen Firmament, but I think I'd like to someday.”"
    );
    p!(
        "At least you know a bit more about your colleague now. The two of you spend some time quietly working together; you catalogue your observations, while the oracle writes their own notes into the journal they carry."
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn vi_firmament_other(_s: &mut State) {
    p!(
        "They nod tersely, wringing their hands. “I'm sure. I'm sure it has... failings. Every city must. I wish...”"
    );
    p!(
        "“Forgive me. I wonder, sometimes, what will happen to us all. There were machines in the times before the saltfall, ones which could perform impossibly complex calculations... so many technologies lost to us. I used to think we were doing nothing meaningful, in our age. No science like the old sciences.” There's a spark of fanaticism in their face. “I know better now.”"
    );
    p!(
        "You politely excuse yourself, and spend the rest of the day cataloguing observations and putting your notes in order."
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn s_city_convo(_s: &mut State) {
    p!(
        "“Rye.” The southernmost city-state, far from Hearth. Known for its agricultural capability, and thus comparatively easy and bountiful quality of life. Unprompted, the oracle volunteers, “I was an artist.”"
    );
    p!(
        "This is unsurprising, considering their general aspect. The meditative way in which they engage with the world. Sometimes they'll contemplate a single rock for minutes on end, turning it over carefully in their hands."
    );

    choice!(
        link!("“What sort of art did you make?”", s_art_details),
        link!("nod and leave well enough alone", s_art_leave)
    );
}

#[ifview]
pub fn s_art_details(s: &mut State) {
    s.relation_oracle += 3;
    p!(
        "They smile. “I painted, mostly. That and small sculptures. Not from life. Imagined landscapes and figures. I've been interested in the possibilities of perception. Light and color. No wonder, right? Since I have visions. I tried to represent them, actually, but I don't think the meaning can be conveyed the way I'd like it to be.”"
    );
    p!(
        "“It wasn't... sustainable for me, after a time. Perhaps the seeing is my stronger skill. I'd like to return to art, though. Someday.”"
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn s_art_leave(_s: &mut State) {
    p!(
        "You spend the rest of the day on your own, cataloguing observations and putting your notes in order."
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn s_exile_talk(s: &mut State) {
    s.used_occipit = true;
    p!(
        "Somehow the topic of exile comes up in your conversation; maybe the bitter cold, maybe the absence of human structures, brings up the idea."
    );
    p!(
        "Exile is effectively a death sentence, used by city-states in lieu of executing their precious populations. Firmament, controversially, was the last to adopt it over actual capital punishment. But the result is the same: a person sent out alone beyond the bounds of a city, without shelter, usually claimed within a day by lethal temperatures."
    );
    p!(
        "“I knew a man who was exiled,” the oracle says. “I had worked with him before then. They exiled him because they said he brought someone back to life.”"
    );
    choice!(link!("“What?”", s_occipit_story));
}

#[ifview]
pub fn s_occipit_story(_s: &mut State) {
    p!(
        "The oracle settles, with a languid motion, to sit more comfortably. “For a time I lived in an apartment in central Rye. I was an artist. Sometimes I modelled for other people, too. Rye center is old and beautiful, but it's so busy. So loud. And I had had a conflict with the owner of the building I was living in. I had to get away for some time.”"
    );
    p!(
        "To own a building. To rent it out to someone else. The concept is foreign to you, but you do know of these things. Neither Hearth nor Firmament would permit such a system."
    );
    p!(
        "“So I went to work at a farm, on one of the outskirts... There was a young man I met there. He told me that the work he used to do was about the human body, a kind of medical science... But then he came to work one day seeming frightened, like he was trying to hide. The next day he was gone. I heard from someone else that he had taken a body—not his family, or even anyone he knew—and brought the person to life somehow. I don't understand it. I don't know why he would be punished for that. I wish he hadn't been killed.”"
    );
    p!(
        "They look up at you, then away. “You know, the other interpreter—the one back at the Observational Society? He reminded me of that young man.”"
    );

    choice!(
        link!("“That's... a strange story.”", s_exile_strange),
        link!(
            "“Nothing can bring back the dead. How superstitious are the people of Rye?”",
            s_exile_skeptical
        )
    );
}

#[ifview]
pub fn s_exile_strange(_s: &mut State) {
    p!(
        "They nod, though they seem lost in thought. “Isn't it? I don't know anything more, like what happened to the person. I don't know. I think he really did do it.”"
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn s_exile_skeptical(s: &mut State) {
    s.relation_oracle -= 3;
    p!(
        "The oracle frowns and does not respond. A moment passes. You know by now that they're a fantastically lenient person. It's something of a shock to finally encounter their outright disapproval."
    );
    p!(
        "“Hm. Well, I told you what I know. It's true, though. They really did exile him for that. I don't know what happened to him. He must have died out there.”"
    );
    p!(
        "They turn their attention away from you. There is a tension that slowly disperses during the course of the afternoon."
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn walker_convo_coldsnap(s: &mut State) {
    match s.walker {
        Walker::A => {
            s.used_firmament_convo = true;
            p!(
                "The walker has questions for you, it seems. “I've been wondering—you're from Firmament, I hear. But you were living and working in Hearth. Some story there, hm? What's that like for you—have you left Firmament behind entirely?”"
            );

            choice!(
                link!("“There's a lot about it that I miss.”", walker_a_miss),
                link!("“I'm loyal only to Hearth.”", walker_a_loyal),
                link!("“I'd... rather not say.”", walker_a_quiet)
            );
        }
        _ => {
            s.used_history_ques = true;
            p!(
                "The walker has questions for you, it seems. “I've been meaning to ask you why you would come all the way out here, looking for a dead city.”"
            );

            choice!(
                link!(
                    "“For our past. We have to understand history.”",
                    walker_t_past
                ),
                link!(
                    "“For our future. Maybe we can fix things.”",
                    walker_t_future
                ),
                link!("“The idea wouldn't leave me alone.”", walker_t_obsessed)
            );
        }
    }
}

#[ifview]
pub fn walker_a_miss(s: &mut State) {
    s.relation_walker -= 3;
    p!(
        "He raises his eyebrows. “Hm. Well, I won't cast judgement on you for that. Did they really treat you that well, or do interpreters have some kind of special privilege in there? I'm inclined to think you're closer to the top of the food chain.”"
    );
    p!(
        "You get the sense that he doesn't expect you to answer. You hear him mutter: “Bunch of ideologues.”"
    );
    p!(
        "You spend the rest of the day alone, cataloguing observations and putting your notes in order."
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn walker_a_loyal(s: &mut State) {
    s.relation_walker += 1;
    p!(
        "He nods slowly, evidently pleased with your answer. “Not that I've anything against it, but I wouldn't choose to live in a kamis state either. Good on you for making your way out of that nest of ideologues.”"
    );
    p!(
        "His words are intended as a friendly jibe, as far as you can tell. But the conversation dwindles after that, and you spend the rest of the day alone, cataloguing observations and putting your notes in order."
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn walker_a_quiet(s: &mut State) {
    s.relation_walker -= 2;
    p!(
        "“You probably think Hearth lacks some things. Maybe it's still strange to you after coming from such an ideological society... I think it's better to have a sense of individuality, though. And it seems like you've taken to it. The way you command this expedition, I'd think of you as a leader, rather than a follower.”"
    );
    p!(
        "His words are intended as a friendly jibe, as far as you can tell. But the conversation dwindles after that, and you spend the rest of the day alone, cataloguing observations and putting your notes in order."
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn walker_t_past(_s: &mut State) {
    p!(
        "The saltwalker nods. “Makes sense. You might well get what you want, if you're right about the objective. If we make it up there... I can't imagine the sort of things you'll find, but maybe you have an idea. Then we'll be part of history ourselves.”"
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn walker_t_future(s: &mut State) {
    match s.walker {
        Walker::T => {
            p!(
                "She chuckles dryly. “How optimistic. I can't say I share that feeling... but best of luck to you.”"
            );
        }
        _ => {
            p!(
                "He nods slowly. “You're younger than I am. Maybe your generation is more optimistic. Well, I wish you luck.”"
            );
        }
    }
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn walker_t_obsessed(_s: &mut State) {
    p!(
        "The saltwalker nods slowly, looking troubled. “I've seen many like you. Be sure to keep in control of yourself, as best as you can. Sometimes the wrack snares your mind.” A brief smile. “But I'm sure you're smarter than most, hm?”"
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn relax_on_your_own(s: &mut State) {
    s.dread = (s.dread - 3).max(0);
    p!(
        "The wailing voice of the wind rattles your tent, but as long as you stay dressed in some of your thermal layers, the outside conditions can't disturb you. You spend a long time sleeping, curled close to the warmth of your ",
        tun!("chemical lamp", chemical_lamp_inspect),
        "."
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}

#[ifview]
pub fn chemical_lamp_inspect(s: &mut State) {
    p!(
        "The lamp's design is utilitarian: made mostly of dull unpainted metal, with a handle set at the top. It casts a bright white light from a cylindrical glass chamber, surrounded by sturdy rods to prevent breakage. The chamber also gives off warmth, slowly and steadily. Every evening, you feed it a little puck of compressed chemical matter, pushing the battery into a slot within the base of the lamp. It accepts it and the slot clicks shut. It acts like some type of battery; you're not a chemist, so you can't name the series of reactions which turn battery and water into brilliance."
    );
    p!(
        "Every morning, after the light has died out along with the need for light, you pull out a sort of drawer on the other side of the base of the lamp. At that point, it is filled with a dark yellowish fluid, which you dump out onto the ice. It reminds you of metabolic waste; that's probably apt, whatever reaction is happening inside the lamp, and anyways the comparison is obvious. Sometimes the lamp needs water to be poured into a third chamber, if it starts flickering."
    );

    if !s.walker.is_dead() {
        p!("The saltwalker, more familiar with the device, does this.");
    }

    choice!(back!("What were you thinking of, again?"));
}

#[ifview]
pub fn keep_up_studies(_s: &mut State) {
    p!(
        "There's not much observation you can safely do while the temperatures are so hazardous. You don't dare venture far from camp. But the wind is high; it might be a good time to take a sample of ",
        link!("airborne particulates.", particulate_matter_1)
    );
}

#[ifview]
pub fn particulate_matter_1(_s: &mut State) {
    p!(
        "You use a fine filter to trap the airborne stuff, then deposit it onto a glass slide for examination. Most of it is merely jagged-edged dust, crystals of salt and other chemicals, bits of stone. High in silica, though you're no geologist."
    );
    p!(
        "But you can see, in a scattered few particles, a pattern like a lacy lattice. It looks eroded and broken, no doubt by years spent tumbling around in the wind. What created these microscopic structures? Surely they're not just some coincidence of iceborne minerals; they almost resemble coral."
    );
    p!(
        "The cold snap abates the next ",
        link!("morning.", crate::saltwrack::chap3::day_eight)
    );
}
