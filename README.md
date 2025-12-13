# IFEngine [![Crates.io](https://img.shields.io/crates/v/ifengine)](https://crates.io/crates/ifengine)

IFEngine is a rust framework for writing interactive fiction.
Other projects in this space include [Twine](https://klembot.github.io/chapbook/) and [Inkle](https://github.com/inkle/ink).

The goal is to enjoy an effortless writing experience, together with all the benefits of the Rust ecosystem.

## [Example](https://ifengine.netlify.app/)


story credit: https://antemaion.itch.io/saltwrack (Sorry I haven't asked permission yet as im just using it for testing this project is still just experimental!)

## Why Rust?

it's cool i like it (todo).

## Features
- Story analysis: Travel your story through the [API](https://docs.rs/ifengine/latest/ifengine/run/index.html). Generate a [graph](https://ifengine.netlify.app/#graph)!
- Nice syntax: Weave together your story elements and code with intuitive [macros](#docs).
- Powerful state management: The full power of the rust language at your behest.
- Navigable content: Stories link and compose together through elements like [link](#https://docs.rs/ifengine/latest/ifengine/elements/macro.link.html), [tunnel](#https://docs.rs/ifengine/latest/ifengine/elements/macro.tun.html), and [weave](#https://docs.rs/ifengine/latest/ifengine/elements/macro.weave.html). Jump between them using `Go to definition`.
- Immediate mode page execution coupled with persisted state enables writing generator-style functions to produce views.


# Guide
0. **Initialize your project**

```shell
mkdir -p ifproject
cd ifproject
```

1. **Download your frontend template**

```
git clone git@github.com:Squirreljetpack/ifengine.git
mv ifengine/egui .
rm -rf ifengine
```
or with [zsh-dl](https://github.com/Squirreljetpack/zsh-dl/):
```
dl https://github.com/Squirreljetpack/ifengine/tree/main/egui
```
Currently, your options are: [egui](./egui) .. and nothing else.

2. **Make a few changes**

`egui/Cargo.toml`:
```toml
[package]
# change me

[dependencies]
# ifengine
ifengine = { path = "../ifengine" } # remove me
story = { path = "../story" }
```

3. **Create your library**
```shell
cargo init --lib story
cd story
cargo add ifengine

# ifproject/
# ├── egui
# └── story
#     ├── Cargo.toml
#     └── src
#         └── lib.rs
```

4. **Write your story** (See: [example](./story/src/saltwrack/chap1.rs), [elements](https://docs.rs/ifengine/latest/ifengine/elements/index.html))

`story/src/lib.rs`:
```rust
pub mod chap1;

pub type Game = ifengine::Game<State>;
pub fn new() -> Game {
    ifengine::Game!(chap1::p1)
}

// your game state
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct State {
    pub name: String,
}
```

`story/src/chap1.rs`:
```rust
#[allow(unused_imports)]
use ifengine::{
    elements::{
        ChoiceVariant::*,
        choice,
        click,
        dp,
        h,
        choice,
        mchoice,
        p,
    },
    utils::MaskExt,
    GO,
    ifview,
    link,
};
use crate::{State};

#[ifview]
pub fn p1(s: &mut State) {
    h!("LET ME TELL YOU WHY I GOT OUT OF BED THIS MORNING FIRST I WOKE UP FEELING A LITTLE HUNGRY AND THIN WHEN THERE CAME THIS SOUND WHICH SOUNDED LIKE NOTHING I HAD EVER HEARD BEFORE THAT AT FIRST I THOUGHT IT WAS MY OLD ENGLISH TEACHER MUMBLING THROUGH THE RADIATOR BUT THEN I REMEMBERED HE HAD GONE AWAY TO QUEBEC WITH HIS DOG AND WOULD NOT KNOW WHERE I LIVED IN REAL LIFE WHEN I HEARD IT AGAIN IT SOUNDED LIKE MORE CLEARLY LIKE A GROAN AND A HISSING THAT I STARTED TO BEGIN TO HAVE A VERY BAD FEELING ABOUT IT WHICH MIGHT HAVE RELATED TO THE FACT THAT I HAD BEEN DEPRESSED ABOUT HUMANITY AND THE STATE OF THE WORLD AS A WHOLE RECENTLY THINKING ABOUT THE CONDITION OF MEN WITHOUT PROSPECTS AND THE CAPTURE OF DECENTRALIZED CURRENCY BY REGULATORY BODIES, BUT OTHERWISE MIGHT HAVE BEEN JUST MY SENSITIVITY OF CHARACTER, WHICH IN EITHER CASE MUST HAVE REACHED A CRITICAL POINT OVER THE COURSE OF THE PREVIOUS NIGHT, CAUSING ME TO SPONTANEOUSLY SHIT THE BED.
        I RECEIVED THE MESSAGE OF MY SUBCONSCIOUS WITH GREAT ALACRITTY, WHICH IS TO SAY I GOT OUT OF BED STRAIGHTAWAY, AND MADE IMMEDIATELY FOR MY WARDROBE. THERE COULD NOT BE A MOMENT TO WASTE -- NOT EVEN ON CUFFING MY SLEEVES AND LEGGINGS, WHICH I USUALLY LIKED TO MAKE SURE OF, BUT THE EXHORTATIONS OF MY INTERNAL SELF HAD BEEN MANIFESTLY CLEAR THIS MORNING, AND SO I SETTLED FOR PULLING MY JACKET OVER MY SLEEVES AND MY NIGHT SHIRT OUT FROM BENEATH MY COLLAR AS I CHANGED THE SHIRT FOR A BANANA BY THE DINING TABLE AND RUSHED OUT THE DOOR HOLDING THE BANANA IN ONE HAND AND A WAD OF TOILET PAPER IN THE OTHER, SPARING A FINGER TO HOOK THE DOOR SHUT BEHIND ME ON MY WAY OUT. THERE WAS NO TIME TO LOCK IT -- THE BEST I COULD DO IT FOR WAS A NOISY TWIST.
        AS LUCK WOULD HAVE IT, I TURNED JUST IN TIME TO SEE MY NEIGHBOR BESIDE ME, POCKETING HIS OWN KEY. HE LOOKED LIKE HE HAD JUST FINISHED CLOSING HIS OWN DOOR, BUT NOT BEING SURE HOW MUCH HE HAD ACTUALLY SEEN, I THOUGHT IT PRUDENT TO GREET HIM. HE LOOKED TO ME WITH SOME SURPRISE, BUT I SMILED AT HIM, AND NODDED TOWARD MY OWN DOOR. \"THERE, NICELY LOCKED\", I SAID, AS BRIGHTLY AS I COULD. HE SEEMED TO STARE AT ME FOR A WHILE, THEN AFTER A WHILE GAVE ME A SLOW NOD BACK. I THINK HE UNDERSTOOD, SO I RUSHED RIGHT AWAY FOR THE LIFT LOBBY. 
        THE SUN WAS BRIGHT FROM THE STAIRWELL, AND A COOL MORNING BREEZE WAFTED A PLASTIC BAG IN A GRACEFUL BALLET ACROSS THE EASTERN WINDOW. IT WAS A BEAUTIFUL MORNING, BUT ONE WHICH I COULD SPARE NO THOUGHT FOR. I PRESSED ALL THE ELEVATOR BUTTONS THEN RAN INTO THE FIRE ESCAPE. FOUR FLIGHTS OF STAIRS DOWN LATER, I EMERGED AT LAST THROUGH A LONG DARKNESS INTO A SEARING LIGHT OF DAY. I WAS STILL THIRSTY AND MY BOWELS WERE STILL LOOSE AND I HAD FORGOTTEN MY WALLET AND KEYCARD ON THE TOP SHELF BY MY DOOR BUT THERE WOULD BE NO TURNING BACK NOW.
        THE WORLD WAS WAITING FOR THE LEVER WHICH WOULD TILT IT...")
    p!(link!("BEGIN", p3));
}
```

5. **Launch!**

```shell
cd egui
trunk serve
```

# Docs

- [elements](https://docs.rs/ifengine/latest/ifengine/elements/index.html)
- [macros](https://docs.rs/ifengine/latest/ifengine/index.html)

>[!NOTE]
> To use this library, you write functions which produce [`Responses`](https://docs.rs/ifengine/latest/ifengine/core/enum.Response.html), eventually resolving to a [`View`](https://docs.rs/ifengine/latest/ifengine/view/struct.View.html). The view corresponding to the current game state is retrieved by calling [`Game::view`](https://docs.rs/ifengine/latest/ifengine/core/struct.Game.html#method.view).
>
> A view is a sequence of [`Objects`](https://docs.rs/ifengine/latest/ifengine/view/enum.Object.html) which you can attach by calling the provided [elements and macros](https://docs.rs/ifengine/latest/ifengine/elements/index.html) within a function decorated by [`#[ifview]`](https://docs.rs/ifengine/latest/ifengine/attr.ifview.html).


# Contributions
Ideas and contributions welcome. There is a rough [todo](./TODO.md) as well.