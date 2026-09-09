use super::State;
use ifengine::{
    elements::{
        EMBED, alts, back, choice, click, count, dchoice, dynamic_choice, fresh, img, link,
        mchoice, p, page_dbg, replace, s, text,
    },
    ifview,
};

#[derive(Clone)]
enum DChoices {
    A,
    B,
    C,
}

#[ifview]
pub fn rainy_day(state: &mut State) {
    p!("text1", "text2");

    replace!((77), "The ancient lock is [[sealed]].");

    if replace!(
        "I'm a wanted criminal on the run from the law and I stumbled here after a failed jump to escape the Zubvian Planetary Police.",
        "Obviously you're not going to tell the truth. Good thing you've got your cover story ready."
    ) {
        choice! {
            s!("I'm a treasure hunter. I search the Galaxy for long-dead civilizations and the things they left behind.").cls("in-500") => {
                state.job = Some("treasure hunter".to_string());
                "I'm a treasure hunter. I search the Galaxy for long-dead civilizations and the things they left behind."
            },
            s!("I'm a traveling merchant. I visit new planets looking for wares to buy and sell. Would you like to buy a fine Darlinian leather jacket?").cls("in-1000") => {
                state.job = Some("merchant".to_string());
                "I'm a traveling merchant. I visit new planets looking for wares to buy and sell. Would you like to buy a fine Darlinian leather jacket?"
            },
            s!("Well, I'm not exactly a tourist, but I am a wanderer. I jump around from system to system looking for new sights and experiences. The stars in this sector of space are absolutely beautiful.").cls("in-1500") => {
                state.job = Some("wanderer".to_string());
                "Well, I'm not exactly a tourist, but I am a wanderer. I jump around from system to system looking for new sights and experiences. The stars in this sector of space are absolutely beautiful."
            },
        };

        if let Some(job) = &state.job {
            p!("Job: {job}");
        }
    }

    text!(
        link!("next", sunny_day),
        s!(" up is: "),
        alts!(["alt1", "alt2", "alt3"], Shuffle)
    );

    choice! {
        (64),
        "1" => "Chose 1",
        "2" => {
            "Chose 2"
        },
    };

    EMBED!(weather_station);

    fresh!(|| {
        dbg!("hello");
    });

    let choices = vec![
        (DChoices::A, "A"), //
        (DChoices::B, "B"),
        (DChoices::C, "C"),
    ];

    if let Some(x) = dynamic_choice!(choices.clone()) {
        match x {
            DChoices::A => {
                dbg!("A handled");
            }
            DChoices::B => {
                dbg!("B handled");
            }
            DChoices::C => {
                state.show_modal = true;
                state.show_popup = true;
            }
        }
    };

    if state.show_modal || state.show_popup {
        EMBED!(rainy_popup :: "popup");
    }

    img!(
        "https://thumb.wikimedia.org/wikipedia/commons/thumb/b/b6/SIPI_Jelly_Beans_4.1.07.tiff/lossy-page1-250px-SIPI_Jelly_Beans_4.1.07.tiff.jpg",
    );

    let dchoice_items = ["Option A", "Option B", "Option C"];
    dchoice!(
        dchoice_items,
        0 => {
            dbg!("A handled");
        }
        1 => {
            dbg!("B handled");
        }
        _ => {
            dbg!("C handled");
        }
    );

    mchoice! {
        (6),
        "1" => {
            // eprintln!("1 clicked")
        },
        "2" => {
            // eprintln!("2 clicked")
        },
    };

    // .insert("hi".into(), "bye".into());
}

#[ifview]
pub fn sunny_day(_: &mut State) {
    p!("sunny");

    text!(
        link!("next", rainy_day),
        "test",
        count!(|n: u64| n.to_string())
    );
}

#[ifview]
pub fn weather_station(_: &mut State) {
    p!("--- Weather Station (Embedded) ---");
    choice! {
        "Check barometer" => "The needle is falling rapidly; barometric pressure is 982 hPa.",
        "Inspect wind vane" => "The vane spins wildly before locking north-northeast.",
    };
    p!(link!("View historical sensor logs", sensor_logs));
}

#[ifview]
pub fn sensor_logs(_: &mut State) {
    p!("--- Historical Sensor Logs Archive ---");
    p!("Sensor record: anomalous salt squalls recorded in years 188, 204, and 214.");
    p!(back!("Back to main weather station"));
}

#[ifview]
pub fn rainy_popup(state: &mut State) {
    let dismiss = click!("Dismiss", {
        state.show_modal = false;
        state.show_popup = false;
    });

    if state.show_modal || state.show_popup {
        p!("--- Popup Notice ---");
        p!("Atmospheric disturbance detected in sector 7.");
        p!(dismiss);
    }
}
