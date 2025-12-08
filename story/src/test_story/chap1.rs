use ifengine::elements::img;
#[allow(unused_imports)]
use ifengine::{
    elements::{alts, choice, count, dchoice, dynamic_choice, text, mchoice, fresh, p, page_dbg},
    ifview, link
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

    text!(
        link!("next", sunny_day),
        alts!(["alt1", "alt2", "alt3"], Shuffle)
    );

    choice! {
        (64),
        "1" => "Chose 1",
        "2" => {
            "Chose 2"
        },
    };

    // next!(sunny_day);

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

    img!("https://upload.wikimedia.org/wikipedia/commons/thumb/b/b6/SIPI_Jelly_Beans_4.1.07.tiff/lossy-page1-256px-SIPI_Jelly_Beans_4.1.07.tiff.jpg",);

    dchoice!(
        choices,
        DChoices::A => {
            dbg!("A handled");
        }
        DChoices::B => {
            dbg!("B handled");
        }
        DChoices::C => {
            todo!()
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
