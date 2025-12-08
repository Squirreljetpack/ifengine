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
    h!("LET ME TELL YOU WHY I GOT OUT OF BED THIS MORNING ITS REAL INTERESTING FIRST THERE WAS A BUG ON THE WALL I THOUGHT I SAW AND THEN", 3);
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