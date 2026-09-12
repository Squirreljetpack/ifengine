#![allow(dead_code)]
use std::{borrow::Cow, collections::HashSet};

mod chap1;

pub type Game = ifengine::Game<State>;
pub fn new() -> Game {
    ifengine::Game!(chap1::p1)
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct State {
    pub days: usize,
    pub rations: usize,
    pub miles: usize,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Companion {
    pub name: Cow<'static, str>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Part1 {
    pub seen: HashSet<String>,
}
