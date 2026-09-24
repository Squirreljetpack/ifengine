use super::State;
use ifengine::{
    elements::{
        EMBED, alts, back, choice, click, count, dchoice, dynamic_choice, fresh, img, link,
        mchoice, p, repl, replace, s, l
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
pub fn rainy_day(s: &mut State) {
    p!("word ", "1");

    if replace!((1), "The ancient lock is [[sealed]].") {
        if replace!((1), "The doorway lies [[open]].") {
            replace!((1), "Stick your hand [[inside]].");
        }
    }

    replace!(
        "The ancient lock is [[sealed]].",
        repl!(
            "The doorway lies [[open]].",
            repl!("Stick your hand [[inside]].", "AAAAH!")
        )
    );

    p!(
        "Well, never mind all that. What brings you out here in the middle of nowhere? We don't exactly get a lot of tourists around these parts."
        ::
        ":Solis"
    );

    if replace!(
        "I'm a wanted criminal on the run from the law and I stumbled here after a failed jump to escape the Zubvian Planetary Police.",
        l!("Obviously you're not going to tell the truth. Good thing you've got your cover story ready.").cls("in-750")
    ) {
        choice! {
            s!("I'm a treasure hunter. I search the Galaxy for long-dead civilizations and the things they left behind.").cls("in-2000") => |l| {
                s.job = Some("treasure hunter".into());
                l
            },
            s!("I'm a traveling merchant. I visit new planets looking for wares to buy and sell. Would you like to buy a fine Darlinian leather jacket?").cls("in-2500") => |l| {
                s.job = Some("merchant".into());
                l
            },
            s!("Well, I'm not exactly a tourist, but I am a wanderer. I jump around from system to system looking for new sights and experiences. The stars in this sector of space are absolutely beautiful.").cls("in-3000") => |l| {
                s.job = Some("wanderer".into());
                l
            },
        };

        if let Some(job) = &s.job {
            p!("Job: {job}");
        }
    }

    let weathers = ["sunny", "cloudy", "rainy"];
    p!(
        link!("tomorrow", sunny_day),
        s!(" will be: "),
        alts!(weathers, Shuffle, |idx| {
            s.weather = weathers[idx].to_string();
        })
        :: "m-0"
    );

    p!(alts!(["Click once", "Done"], Stop) :: "m-0");

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
                s.popup = true;
            }
        }
    };

    if s.popup {
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
        "1" => "1 clicked",
        "2" => "2 clicked",
    };
}

#[ifview]
pub fn sunny_day(state: &mut State) {
    p!(state.weather.clone());

    p!(
        "go back to ",
        link!("yesterday", rainy_day),
        "... or ",
        count!(|n: u64| s!("click me: {n}"))
        :: "m-0"
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
        state.popup = false;
    });

    if state.popup {
        p!("--- Popup Notice ---");
        p!("Atmospheric disturbance detected in sector 7.");
        p!(dismiss);
    }
}
