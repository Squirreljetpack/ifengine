use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod game_wrapper;
mod render;

use game_wrapper::ActiveGame;

const DEFAULT_STATE_FILE: &str = ".ifengine_state.json";

struct CliConfig {
    state_file: PathBuf,
    json: bool,
    subcommand: String,
    args: Vec<String>,
}

fn print_help() {
    println!(
        r#"ifengine-cli: Minimal interactive frontend and playtesting harness for ifengine stories

USAGE:
  ifengine-cli [OPTIONS] <SUBCOMMAND> [ARGS...]

SUBCOMMANDS:
  new [story]             Start a new game (default: chant; also supports forest, saltwrack, test_story)
                          If story is omitted and a save exists, restarts that story.
  choose <id_or_text>     Interact with an element by index [0, 1...] or matching text
  view                    Display current story view and available choices
  action <name> [args...] Perform a global state action (e.g. inventory, money, lives, give, inspect, set)
                          Aliases: act, do
  save <path>             Export current game state to a custom path
  load <path>             Load game state from a custom path
  simulate [story]        Run headless simulation exploring story branches (alias: sim)
  help                    Print this help information

OPTIONS:
  --state <path>          Path to state file (default: .ifengine_state.json or IFENGINE_STATE env)
  --json                  Output raw structured JSON instead of formatted text
"#
    );
}

fn parse_cli_args() -> Result<CliConfig, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut state_file = std::env::var("IFENGINE_STATE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_STATE_FILE));
    let mut json = false;

    let mut filtered = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--json" {
            json = true;
            i += 1;
        } else if arg == "--state" {
            if i + 1 >= args.len() {
                return Err("Missing argument for --state".into());
            }
            state_file = PathBuf::from(&args[i + 1]);
            i += 2;
        } else if arg == "--help" || arg == "-h" {
            filtered.push("help".to_string());
            i += 1;
        } else {
            filtered.push(arg.clone());
            i += 1;
        }
    }

    if filtered.is_empty() {
        filtered.push("view".to_string());
    }

    let subcommand = filtered.remove(0);
    Ok(CliConfig {
        state_file,
        json,
        subcommand,
        args: filtered,
    })
}

fn load_state(path: &Path) -> Result<ActiveGame, String> {
    if !path.exists() {
        return Err(format!(
            "No game state found at '{}'. Please start a new game first using 'ifengine-cli new [story]'.",
            path.display()
        ));
    }
    let data = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read state file '{}': {e}", path.display()))?;
    serde_json::from_str(&data)
        .map_err(|e| format!("Failed to deserialize game state from '{}': {e}", path.display()))
}

fn save_state(path: &Path, game: &ActiveGame) -> Result<(), String> {
    let data = serde_json::to_string_pretty(game)
        .map_err(|e| format!("Failed to serialize game state: {e}"))?;
    fs::write(path, data)
        .map_err(|e| format!("Failed to write state file '{}': {e}", path.display()))?;
    Ok(())
}

fn output_game_view(game: &mut ActiveGame, json: bool, notice: Option<&str>) -> Result<(), String> {
    let story = game.story_name();
    let status = game.status_summary();
    let view = game
        .view()
        .map_err(|e| format!("Error rendering page view: {e:?}"))?;

    if json {
        let mut val = render::render_view_to_json(story, &status, &view);
        if let Some(msg) = notice {
            val["notice"] = serde_json::Value::String(msg.to_string());
        }
        println!("{}", serde_json::to_string_pretty(&val).unwrap());
    } else {
        if let Some(msg) = notice {
            println!(">>> {msg}\n");
        }
        let rendered = render::render_view_to_text(story, &status, &view);
        print!("{rendered}");
    }

    Ok(())
}

fn normalize_for_match(s: &str) -> String {
    s.to_lowercase()
        .replace(['’', '‘', '`'], "'")
        .replace(['“', '”'], "\"")
        .trim()
        .to_string()
}

fn main() -> ExitCode {
    let config = match parse_cli_args() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error: {err}");
            return ExitCode::FAILURE;
        }
    };

    match config.subcommand.as_str() {
        "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        "new" => {
            let story_name = if let Some(name) = config.args.first() {
                name.as_str()
            } else if config.state_file.exists() {
                // Restart existing game if state exists
                match load_state(&config.state_file) {
                    Ok(existing) => existing.story_name(),
                    Err(_) => "chant",
                }
            } else {
                "chant"
            };

            let mut game = match ActiveGame::new_story(story_name) {
                Ok(g) => g,
                Err(err) => {
                    eprintln!("Error: {err}");
                    return ExitCode::FAILURE;
                }
            };

            let notice = format!("Started new game of '{story_name}' (saved to {})", config.state_file.display());
            if let Err(err) = output_game_view(&mut game, config.json, Some(&notice)) {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }

            if let Err(err) = save_state(&config.state_file, &game) {
                eprintln!("Error saving state: {err}");
                return ExitCode::FAILURE;
            }

            ExitCode::SUCCESS
        }
        "view" => {
            let mut game = match load_state(&config.state_file) {
                Ok(g) => g,
                Err(err) => {
                    eprintln!("Error: {err}");
                    return ExitCode::FAILURE;
                }
            };

            if let Err(err) = output_game_view(&mut game, config.json, None) {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }

            ExitCode::SUCCESS
        }
        "choose" => {
            let target = match config.args.first() {
                Some(t) => t.trim(),
                None => {
                    eprintln!("Error: 'choose' subcommand requires an interactable index or text. Usage: ifengine-cli choose <id_or_text>");
                    return ExitCode::FAILURE;
                }
            };

            let mut game = match load_state(&config.state_file) {
                Ok(g) => g,
                Err(err) => {
                    eprintln!("Error: {err}");
                    return ExitCode::FAILURE;
                }
            };

            let view = match game.view() {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("Error rendering view: {err:?}");
                    return ExitCode::FAILURE;
                }
            };

            let interactables = view.interactables_sim();
            if interactables.is_empty() {
                eprintln!("Error: No interactable choices or links available on current page '{}'.", view.pageid.0);
                return ExitCode::FAILURE;
            }

            let chosen_index = if let Ok(idx) = target.parse::<usize>() {
                if idx >= interactables.len() {
                    eprintln!("Error: Choice index {idx} out of range (available: 0..{}).", interactables.len());
                    return ExitCode::FAILURE;
                }
                idx
            } else {
                // Match by text content with normalized quotes/apostrophes
                let norm_target = normalize_for_match(target);
                let exact_match = interactables.iter().position(|e| normalize_for_match(&e.content()) == norm_target);
                let substring_match = interactables.iter().position(|e| normalize_for_match(&e.content()).contains(&norm_target));

                match exact_match.or(substring_match) {
                    Some(idx) => idx,
                    None => {
                        eprintln!("Error: Could not find interactable matching '{target}'. Available choices:");
                        for (i, item) in interactables.iter().enumerate() {
                            eprintln!("  [{i}] {}", item.content().trim());
                        }
                        return ExitCode::FAILURE;
                    }
                }
            };

            let chosen_interactable = interactables[chosen_index];
            let chosen_label = chosen_interactable.content().to_string();

            if let Err(err) = game.interact(chosen_interactable) {
                eprintln!("Error executing interaction: {err:?}");
                return ExitCode::FAILURE;
            }

            let notice = format!("Chose [{chosen_index}]: '{chosen_label}'");
            if let Err(err) = output_game_view(&mut game, config.json, Some(&notice)) {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }

            if let Err(err) = save_state(&config.state_file, &game) {
                eprintln!("Error saving state: {err}");
                return ExitCode::FAILURE;
            }

            ExitCode::SUCCESS
        }
        "action" | "act" | "do" => {
            let action_name = match config.args.first() {
                Some(a) => a.clone(),
                None => {
                    eprintln!("Error: 'action' subcommand requires an action name (e.g. inventory, money, set).");
                    return ExitCode::FAILURE;
                }
            };

            let action_args = config.args[1..].to_vec();

            let mut game = match load_state(&config.state_file) {
                Ok(g) => g,
                Err(err) => {
                    eprintln!("Error: {err}");
                    return ExitCode::FAILURE;
                }
            };

            let result_msg = match game.apply_action(&action_name, &action_args) {
                Ok(msg) => msg,
                Err(err) => {
                    eprintln!("Action error: {err}");
                    return ExitCode::FAILURE;
                }
            };

            if let Err(err) = output_game_view(&mut game, config.json, Some(&result_msg)) {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }

            if let Err(err) = save_state(&config.state_file, &game) {
                eprintln!("Error saving state: {err}");
                return ExitCode::FAILURE;
            }

            ExitCode::SUCCESS
        }
        "save" => {
            let dest = match config.args.first() {
                Some(p) => PathBuf::from(p),
                None => {
                    eprintln!("Error: 'save' subcommand requires a destination path. Usage: ifengine-cli save <path>");
                    return ExitCode::FAILURE;
                }
            };

            let game = match load_state(&config.state_file) {
                Ok(g) => g,
                Err(err) => {
                    eprintln!("Error: {err}");
                    return ExitCode::FAILURE;
                }
            };

            if let Err(err) = save_state(&dest, &game) {
                eprintln!("Error saving state: {err}");
                return ExitCode::FAILURE;
            }

            println!("Game state successfully saved to '{}'.", dest.display());
            ExitCode::SUCCESS
        }
        "load" => {
            let src = match config.args.first() {
                Some(p) => PathBuf::from(p),
                None => {
                    eprintln!("Error: 'load' subcommand requires a source path. Usage: ifengine-cli load <path>");
                    return ExitCode::FAILURE;
                }
            };

            let mut game = match load_state(&src) {
                Ok(g) => g,
                Err(err) => {
                    eprintln!("Error: {err}");
                    return ExitCode::FAILURE;
                }
            };

            if let Err(err) = save_state(&config.state_file, &game) {
                eprintln!("Error saving state: {err}");
                return ExitCode::FAILURE;
            }

            let notice = format!("Loaded game state from '{}'", src.display());
            if let Err(err) = output_game_view(&mut game, config.json, Some(&notice)) {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }

            ExitCode::SUCCESS
        }
        "simulate" | "sim" => {
            let story_name = config.args.first().map(|s| s.as_str()).unwrap_or("chant");
            let game = match ActiveGame::new_story(story_name) {
                Ok(g) => g,
                Err(err) => {
                    eprintln!("Error: {err}");
                    return ExitCode::FAILURE;
                }
            };

            println!("Simulating story '{story_name}'...");
            let report = game.simulate();
            println!("Simulation Complete for '{}':", report.story);
            println!("  Reachable pages recorded: {}", report.total_pages_visited);
            println!("  Branch endpoints reached: {}", report.endpoints);
            if report.errors.is_empty() {
                println!("  Errors encountered: None");
            } else {
                println!("  Errors ({}):", report.errors.len());
                for e in &report.errors {
                    println!("    - {e}");
                }
            }

            ExitCode::SUCCESS
        }
        unknown => {
            eprintln!("Unknown subcommand '{unknown}'. Use 'ifengine-cli help' for usage.");
            ExitCode::FAILURE
        }
    }
}
