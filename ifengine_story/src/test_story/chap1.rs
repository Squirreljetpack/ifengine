use ifengine::{elements::ddchoices, link};
#[allow(unused_imports)]
use ifengine::{
    elements::{alts, choice, count, dchoices, line, mchoice, once, p, page_dbg},
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

    line!(
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

    once!(|| {
        dbg!("hello");
    });

    let choices = vec![
        (DChoices::A, "A"), //
        (DChoices::B, "B"),
        (DChoices::C, "C"),
    ];

    if let Some(x) = dchoices!(choices.clone()) {
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

    ddchoices!(
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

    line!(
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
