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
  play [story]            Launch continuous interactive session (aliases: repl, interactive)
                          Default story: chant (also supports forest, saltwrack, test_story)
  new [story]             Start a new game and display the first page
                          If story is omitted and a save exists, restarts that story.
  choose <index>          Interact with an element by integer index [0, 1...]
  view                    Display current story view and available choices
  action <name> [args...] Perform a global state action (e.g. inventory, money, lives, give, take, set)
                          Aliases: act, do
  save <path>             Export current game state to a custom path
  load <path>             Load game state from a custom path
  simulate [story] [depth] Run headless simulation exploring story branches (alias: sim)
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
    let view = game
        .view()
        .map_err(|e| format!("Error rendering page view: {e:?}"))?;
    let status = game.status_summary();

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

fn perform_choice(game: &mut ActiveGame, chosen_index: usize) -> Result<String, String> {
    let view = game
        .view()
        .map_err(|e| format!("Error rendering view: {e:?}"))?;

    let interactables = view.interactables_flat::<false>();
    if interactables.is_empty() {
        return Err(format!(
            "No interactable choices or links available on current page '{}'.",
            view.pageid.0
        ));
    }

    if chosen_index >= interactables.len() {
        return Err(format!(
            "Choice index {chosen_index} out of range (available: 0..{}).",
            interactables.len()
        ));
    }

    let chosen_interactable = interactables[chosen_index];
    let chosen_label = chosen_interactable.content().to_string();

    game.interact(chosen_interactable)
        .map_err(|e| format!("Error executing interaction: {e:?}"))?;

    Ok(format!("Chose [{chosen_index}]: '{chosen_label}'"))
}

fn run_repl(config: &CliConfig) -> ExitCode {
    use std::io::{self, BufRead, Write};

    let mut game = if config.state_file.exists() {
        match load_state(&config.state_file) {
            Ok(g) => g,
            Err(_) => {
                let story_name = config.args.first().map(|s| s.as_str()).unwrap_or("chant");
                match ActiveGame::new_story(story_name) {
                    Ok(g) => g,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
        }
    } else {
        let story_name = config.args.first().map(|s| s.as_str()).unwrap_or("chant");
        match ActiveGame::new_story(story_name) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Error: {e}");
                return ExitCode::FAILURE;
            }
        }
    };

    println!("=== Welcome to ifengine-cli Interactive Session ===");
    println!("Type an integer choice index. Type 'help' for commands, 'quit' to exit.\n");
    if let Err(e) = output_game_view(&mut game, config.json, None) {
        eprintln!("{e}");
    }

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let line = match lines.next() {
            Some(Ok(l)) => l.trim().to_string(),
            Some(Err(e)) => {
                eprintln!("Input error: {e}");
                break;
            }
            None => break,
        };

        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "quit" | "exit" | "q" => {
                println!("Saving state to '{}' and exiting.", config.state_file.display());
                let _ = save_state(&config.state_file, &game);
                break;
            }
            "help" | "h" | "?" => {
                println!("Interactive Commands:");
                println!("  <index>            Make a choice or follow link");
                println!("  view / look / l    Reprint current page and choices");
                println!("  inv [open|close]   Toggle or view inventory");
                println!("  action <name> ...  Run cheat or state action (lives, money, give, take)");
                println!("  new [story]        Restart game");
                println!("  save [path]        Save progress");
                println!("  load [path]        Load progress");
                println!("  quit / exit / q    Save progress and exit\n");
            }
            "view" | "look" | "l" => {
                let _ = output_game_view(&mut game, config.json, None);
            }
            "inv" | "inventory" => {
                let mode = parts.get(1).copied().unwrap_or("toggle");
                match game.apply_action("inventory", &[mode.to_string()]) {
                    Ok(msg) => {
                        let _ = output_game_view(&mut game, config.json, Some(&msg));
                        let _ = save_state(&config.state_file, &game);
                    }
                    Err(err) => println!("Error: {err}"),
                }
            }
            "new" => {
                let story = parts.get(1).copied().unwrap_or_else(|| game.story_name());
                match ActiveGame::new_story(story) {
                    Ok(new_g) => {
                        game = new_g;
                        let notice = format!("Started new game of '{story}'");
                        let _ = output_game_view(&mut game, config.json, Some(&notice));
                        let _ = save_state(&config.state_file, &game);
                    }
                    Err(e) => println!("Error: {e}"),
                }
            }
            "save" => {
                let path = parts.get(1).map(PathBuf::from).unwrap_or_else(|| config.state_file.clone());
                match save_state(&path, &game) {
                    Ok(()) => println!("Saved to '{}'.", path.display()),
                    Err(e) => println!("Error saving: {e}"),
                }
            }
            "load" => {
                let path = parts.get(1).map(PathBuf::from).unwrap_or_else(|| config.state_file.clone());
                match load_state(&path) {
                    Ok(loaded) => {
                        game = loaded;
                        let notice = format!("Loaded state from '{}'", path.display());
                        let _ = output_game_view(&mut game, config.json, Some(&notice));
                    }
                    Err(e) => println!("Error loading: {e}"),
                }
            }
            "action" | "act" | "do" => {
                if parts.len() < 2 {
                    println!("Usage: action <name> [args...]");
                } else {
                    let name = parts[1];
                    let args: Vec<String> = parts[2..].iter().map(|s| s.to_string()).collect();
                    match game.apply_action(name, &args) {
                        Ok(msg) => {
                            let _ = output_game_view(&mut game, config.json, Some(&msg));
                            let _ = save_state(&config.state_file, &game);
                        }
                        Err(e) => println!("Action error: {e}"),
                    }
                }
            }
            _ => {
                if let Ok(idx) = line.parse::<usize>() {
                    match perform_choice(&mut game, idx) {
                        Ok(notice) => {
                            let _ = output_game_view(&mut game, config.json, Some(&notice));
                            let _ = save_state(&config.state_file, &game);
                        }
                        Err(err) => {
                            println!("Error: {err}");
                        }
                    }
                } else {
                    println!("Invalid input: enter an integer choice index [0, 1...] or a command like 'view', 'inv', 'action', 'quit'.");
                }
            }
        }
    }

    ExitCode::SUCCESS
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
        "play" | "repl" | "interactive" => run_repl(&config),
        "new" => {
            let story_name = if let Some(name) = config.args.first() {
                name.as_str()
            } else if config.state_file.exists() {
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
            let target_str = match config.args.first() {
                Some(t) => t.trim(),
                None => {
                    eprintln!("Error: 'choose' subcommand requires an integer index. Usage: ifengine-cli choose <index>");
                    return ExitCode::FAILURE;
                }
            };

            let chosen_index = match target_str.parse::<usize>() {
                Ok(idx) => idx,
                Err(_) => {
                    eprintln!("Error: 'choose' requires an integer index (e.g. 'ifengine-cli choose 0').");
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

            let notice = match perform_choice(&mut game, chosen_index) {
                Ok(n) => n,
                Err(err) => {
                    eprintln!("{err}");
                    return ExitCode::FAILURE;
                }
            };

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
            let depth = config.args.get(1).and_then(|d| d.parse::<usize>().ok());
            let game = match ActiveGame::new_story(story_name) {
                Ok(g) => g,
                Err(err) => {
                    eprintln!("Error: {err}");
                    return ExitCode::FAILURE;
                }
            };

            let depth_str = depth.map(|d| format!(" (max depth: {d})")).unwrap_or_default();
            println!("Simulating story '{story_name}'{depth_str}...");
            let report = game.simulate(depth);
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
