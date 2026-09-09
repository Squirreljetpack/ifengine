use ifengine::{
    elements::{
        alts, back, choice, count, dchoice, dynamic_choice, fresh, img, link, mchoice, p,
        page_dbg, replace, s, text, EMBED,
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
pub fn rainy_day(_: &mut ()) {
    p!("text1", "text2");

    replace!((77), "The ancient lock is [[sealed]].");

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
                todo!()
            }
        }
    };

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
pub fn sunny_day(_: &mut ()) {
    p!("sunny");

    text!(
        link!("next", rainy_day),
        "test",
        count!(|n: u64| n.to_string())
    );

    // choose!({
    //     "1" => "Chose 1",
    //     "2" => {
    //         "Chose 2"
    //     },
    // });
    // what about Always choices
    // do we also need to track the choice which was last clicked

    // do we want to expose a way to change the page directly instead of through link

    // if x > 0 ... change

    // click -> link ->

    // you might want if choices is done
    // let (chosen [bool], element) = choices! [
    // {
    //     do_something;
    //     choice_render
    // }
    // ]
    // add!(element)
}

#[ifview]
pub fn weather_station(_: &mut ()) {
    p!("--- Weather Station (Embedded) ---");
    choice! {
        "Check barometer" => "The needle is falling rapidly; barometric pressure is 982 hPa.",
        "Inspect wind vane" => "The vane spins wildly before locking north-northeast.",
    };
    p!(link!("View historical sensor logs", sensor_logs));
}

#[ifview]
pub fn sensor_logs(_: &mut ()) {
    p!("--- Historical Sensor Logs Archive ---");
    p!("Sensor record: anomalous salt squalls recorded in years 188, 204, and 214.");
    p!(back!("Back to main weather station"));
}

