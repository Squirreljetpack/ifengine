use serde::{Deserialize, Serialize};

use ifengine::View;
use ifengine_core::run::Interactable;

#[derive(Serialize, Deserialize)]
pub enum ActiveGame {
    Chant(chant::Game),
    Forest(forest::Game),
    Saltwrack(saltwrack::Game),
    TestStory(test_story::Game),
}

pub struct SimReport {
    pub story: &'static str,
    pub total_pages_visited: usize,
    pub endpoints: usize,
    pub errors: Vec<String>,
}

impl ActiveGame {
    pub fn new_story(name: &str) -> Result<Self, String> {
        match name.to_lowercase().as_str() {
            "chant" | "christopher" => Ok(Self::Chant(chant::new())),
            "forest" => Ok(Self::Forest(forest::new())),
            "saltwrack" => Ok(Self::Saltwrack(saltwrack::new())),
            "test" | "test_story" => Ok(Self::TestStory(test_story::new())),
            unknown => Err(format!(
                "Unknown story '{unknown}'. Available stories: chant, forest, saltwrack, test_story"
            )),
        }
    }

    pub fn story_name(&self) -> &'static str {
        match self {
            Self::Chant(_) => "chant",
            Self::Forest(_) => "forest",
            Self::Saltwrack(_) => "saltwrack",
            Self::TestStory(_) => "test_story",
        }
    }

    pub fn view(&mut self) -> Result<View, ifengine::GameError> {
        match self {
            Self::Chant(g) => g.view(),
            Self::Forest(g) => g.view(),
            Self::Saltwrack(g) => g.view(),
            Self::TestStory(g) => g.view(),
        }
    }

    pub fn interact(&mut self, e: Interactable<'_>) -> Result<(), ifengine::GameError> {
        match self {
            Self::Chant(g) => g.interact(e),
            Self::Forest(g) => g.interact(e),
            Self::Saltwrack(g) => g.interact(e),
            Self::TestStory(g) => g.interact(e),
        }
    }

    pub fn status_summary(&self) -> Vec<String> {
        match self {
            Self::Chant(g) => {
                let s = &g.context;
                vec![
                    format!("Lives: {}/9", s.lives),
                    format!("Guineas: {}", s.guineas),
                    format!("Realm: {}", s.current_realm.display_name()),
                    format!("Inventory: {} item(s)", s.inventory.len()),
                    format!("Ghost-Sight: {}", if s.ghost_sight_active { "Active" } else { "Inactive" }),
                    format!("Inventory Modal: {}", if s.inventory_open { "OPEN" } else { "Closed" }),
                ]
            }
            Self::Forest(g) => {
                let s = &g.context;
                vec![
                    format!("Day: {}", s.days),
                    format!("Miles: {}", s.miles),
                    format!("Rations: {}", s.rations),
                ]
            }
            Self::Saltwrack(_) => {
                vec!["Saltwrack Episode".to_string()]
            }
            Self::TestStory(g) => {
                let s = &g.context;
                vec![
                    format!("Weather: {}", s.weather),
                    format!("Miles: {}", s.miles),
                    format!("Rations: {}", s.rations),
                    format!("Job: {}", s.job.as_deref().unwrap_or("None")),
                ]
            }
        }
    }

    pub fn apply_action(&mut self, action: &str, args: &[String]) -> Result<String, String> {
        match self {
            Self::Chant(g) => apply_chant_action(g, action, args),
            Self::Forest(g) => apply_forest_action(g, action, args),
            Self::Saltwrack(_) => Err("Saltwrack does not have custom action cheats configured.".into()),
            Self::TestStory(g) => apply_test_action(g, action, args),
        }
    }

    pub fn simulate(&self) -> SimReport {
        let story = self.story_name();
        let mut visited_count = 0;
        let mut endpoints = 0;
        let errors = Vec::new();

        const MAX_DEPTH: usize = 6;
        match self {
            Self::Chant(g) => {
                let sim = g.simulate(|state| state.depth < MAX_DEPTH);
                for (_id, records) in sim.runs {
                    visited_count += records.0.len();
                    for rec in &records.0 {
                        endpoints += rec.ends.len();
                    }
                }
            }
            Self::Forest(g) => {
                let sim = g.simulate(|state| state.depth < MAX_DEPTH);
                for (_id, records) in sim.runs {
                    visited_count += records.0.len();
                    for rec in &records.0 {
                        endpoints += rec.ends.len();
                    }
                }
            }
            Self::Saltwrack(g) => {
                let sim = g.simulate(|state| state.depth < MAX_DEPTH);
                for (_id, records) in sim.runs {
                    visited_count += records.0.len();
                    for rec in &records.0 {
                        endpoints += rec.ends.len();
                    }
                }
            }
            Self::TestStory(g) => {
                let sim = g.simulate(|state| state.depth < MAX_DEPTH);
                for (_id, records) in sim.runs {
                    visited_count += records.0.len();
                    for rec in &records.0 {
                        endpoints += rec.ends.len();
                    }
                }
            }
        }

        SimReport {
            story,
            total_pages_visited: visited_count,
            endpoints,
            errors,
        }
    }
}

fn apply_chant_action(g: &mut chant::Game, action: &str, args: &[String]) -> Result<String, String> {
    let s = &mut g.context;
    match action.to_lowercase().as_str() {
        "inventory" | "inv" => {
            let mode = args.first().map(|x| x.to_lowercase()).unwrap_or_else(|| "toggle".to_string());
            match mode.as_str() {
                "open" | "show" | "true" | "1" => {
                    s.inventory_open = true;
                    Ok("Inventory modal opened.".into())
                }
                "close" | "hide" | "false" | "0" => {
                    s.inventory_open = false;
                    s.inspected_item_modal = None;
                    s.inspect_action_message = None;
                    Ok("Inventory modal closed.".into())
                }
                "toggle" => {
                    s.inventory_open = !s.inventory_open;
                    if !s.inventory_open {
                        s.inspected_item_modal = None;
                        s.inspect_action_message = None;
                    }
                    Ok(format!("Inventory modal is now {}.", if s.inventory_open { "OPEN" } else { "CLOSED" }))
                }
                other => Err(format!("Unknown mode '{other}'. Use open, close, or toggle.")),
            }
        }
        "money" | "guineas" => {
            let amount: u32 = args.first()
                .ok_or_else(|| "Usage: action money <amount>".to_string())?
                .parse()
                .map_err(|e| format!("Invalid amount: {e}"))?;
            s.guineas = amount;
            Ok(format!("Set guineas to {amount}."))
        }
        "set" => {
            if args.len() < 2 {
                return Err("Usage: action set <field_name> <value>".into());
            }
            let field = args[0].to_lowercase();
            let val = &args[1];
            match field.as_str() {
                "lives" => { s.lives = val.parse().map_err(|e| format!("{e}"))?; }
                "ghost_sight_active" | "ghost_sight" => { s.ghost_sight_active = parse_bool(val)?; }
                "nursery_floorboard_open" => { s.nursery_floorboard_open = parse_bool(val)?; }
                "tacroy_firmed_up" => { s.tacroy_firmed_up = parse_bool(val)?; }
                "dorm_patrol_bypassed" => { s.dorm_patrol_bypassed = parse_bool(val)?; }
                "castle_safe_unlocked" => { s.castle_safe_unlocked = parse_bool(val)?; }
                "eleven_gate_calibrated" => { s.eleven_gate_calibrated = parse_bool(val)?; }
                other => return Err(format!("Field '{other}' not recognized for action set.")),
            }
            Ok(format!("Field '{field}' set to {val}."))
        }
        unknown => Err(format!(
            "Unknown action '{unknown}'. Supported actions: inventory [open|close|toggle], money <amount>, set <field> <val>"
        )),
    }
}

fn apply_forest_action(g: &mut forest::Game, action: &str, args: &[String]) -> Result<String, String> {
    let s = &mut g.context;
    match action.to_lowercase().as_str() {
        "days" => {
            let n: usize = args.first().ok_or("Usage: action days <number>")?.parse().map_err(|e| format!("{e}"))?;
            s.days = n;
            Ok(format!("Days set to {n}."))
        }
        "miles" => {
            let n: usize = args.first().ok_or("Usage: action miles <number>")?.parse().map_err(|e| format!("{e}"))?;
            s.miles = n;
            Ok(format!("Miles set to {n}."))
        }
        "rations" => {
            let n: usize = args.first().ok_or("Usage: action rations <number>")?.parse().map_err(|e| format!("{e}"))?;
            s.rations = n;
            Ok(format!("Rations set to {n}."))
        }
        unknown => Err(format!("Unknown action '{unknown}' for forest. Available: days, miles, rations")),
    }
}

fn apply_test_action(g: &mut test_story::Game, action: &str, args: &[String]) -> Result<String, String> {
    let s = &mut g.context;
    match action.to_lowercase().as_str() {
        "weather" => {
            let w = args.join(" ");
            s.weather = w.clone();
            Ok(format!("Weather set to '{w}'."))
        }
        "miles" => {
            let n: usize = args.first().ok_or("Usage: action miles <n>")?.parse().map_err(|e| format!("{e}"))?;
            s.miles = n;
            Ok(format!("Miles set to {n}."))
        }
        "rations" => {
            let n: usize = args.first().ok_or("Usage: action rations <n>")?.parse().map_err(|e| format!("{e}"))?;
            s.rations = n;
            Ok(format!("Rations set to {n}."))
        }
        unknown => Err(format!("Unknown action '{unknown}' for test_story. Available: weather, miles, rations")),
    }
}

fn parse_bool(s: &str) -> Result<bool, String> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        other => Err(format!("Invalid boolean '{other}', expected true or false")),
    }
}
