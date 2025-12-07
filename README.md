# Ifengine

Ifengine is a rust framework for writing interactive fiction.
Other frameworks in this space include [Twine](https://klembot.github.io/chapbook/) and [Inkle](#https://github.com/inkle/ink).

The goal is to enjoy an effortless writing experience, as well as all the benefits of the Rust ecosystem!

# Why Rust?

TODO

# Guide

1. **Download your frontend template**
```
git clone git@github.com:Squirreljetpack/ifengine.git
mv ifengine/ifengine_egui .
rm -rf ifengine
```
or with [zsh-dl](https://github.com/Squirreljetpack/zsh-dl/):
```
dl https://github.com/Squirreljetpack/ifengine/tree/main/ifengine_egui
```
Currently, your options are: [egui](./ifengine_egui) .. and nothing else.

2. **Create your library**

```shell
cargo init woke_story
cargo add ifengine
```

3. **Write your story** (See: [example](./ifengine_story/src/saltwrack/chap1.rs), [elements](#docs))


```rust
// ----------- woke_story/src/lib.rs -----------

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

```rust
// ----------- woke_story/src/chap1.rs -----------
#[ifview]
pub fn p1(s: &mut State) {
    h!("WHY I GOT OUT OF BED THIS MORNING", 3);
    p!(link!("BEGIN", p3));
}

```

4. **Launch!**
```rust
// ifengine_egui/src/app_type.rs
use my_story::{new, Game}; // change me
```

```shell
cd ifengine_egui
trunk serve
```

# Example
https://ifengine.netlify.app/

story credit: https://antemaion.itch.io/saltwrack (Sorry I haven't asked permission yet as im just using it for testing this project is still just experimental!)

# Features
- Story analysis: Generate a [graph](https://ifengine.netlify.app/#graph) of your story!
- Nice syntax: Weave together your story elements and code with intuitive [macros](#docs).
- Powerful state management: The full power of the rust language at your behest!
- Navigable content: Stories link and compose together through elements like [link](#link), [tunnel](#tunnel), and [weave](#weave). Rust analyze allows you to jump between them with Go to definition.


# Docs

todo