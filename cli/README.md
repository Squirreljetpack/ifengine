# `ifengine-cli`

A terminal frontend, interactive player, and playtesting test-harness for stories built on the [`ifengine`](../ifengine) framework.

It allows human players and automated test scripts to explore branching narratives, test edge cases, trigger debugging cheats, inspect inventory/status states, and run headless branch simulations.

Supported stories out of the box:
- `chant` (*The Lives of Christopher Chant*)
- `forest` (*The Road Through the Forest*)
- `saltwrack` (*Saltwrack Deep*)
- `test_story` (*Engine test harness*)

---

## Quick Start

### 1. Interactive Play (REPL Mode)
Launch a continuous terminal session where you make choices using index numbers or text:
```bash
# Launch default story (chant)
cargo run --bin ifengine-cli -- play

# Or specify a different story
cargo run --bin ifengine-cli -- play forest
```

Inside the interactive session:
```text
> 0                     # Choose option by integer index
> 1                     # Choose next option
> inv                   # Toggle inventory pocket view
> action money 100      # Cheat: grant 100 guineas
> action items          # List all items in inventory and worn
> save my_save.json     # Save game progress
> help                  # List available interactive commands
> quit                  # Save state and exit
```

### 2. One-Shot / Scriptable CLI Commands
Every interaction can be executed as a standalone command. State is persisted between invocations in `.ifengine_state.json` (or via `--state <path>`):

```bash
# 1. Start a new game of Chant
cargo run --bin ifengine-cli -- new chant

# 2. View current page, status, and choices
cargo run --bin ifengine-cli -- view

# 3. Make a choice by integer index
cargo run --bin ifengine-cli -- choose 0
```

---

## Command Reference

### Global Options
- `--state <path>`: Path to game state file (default: `.ifengine_state.json`). Also configurable via the `IFENGINE_STATE` environment variable.
- `--json`: Outputs structured JSON instead of formatted terminal text. Ideal for automated tools and assertions.
- `-h`, `--help`, `help`: Displays CLI usage.

---

### Subcommands

#### `play [story]` *(Aliases: `repl`, `interactive`)*
Launches a continuous interactive loop. Loads existing state if present, or initializes the requested story (defaults to `chant`). Automatically saves state on exit or after actions.

#### `new [story]`
Initializes a new game session and prints the starting page. If `story` is omitted, resets the story currently saved in state.

```bash
cargo run --bin ifengine-cli -- new chant
cargo run --bin ifengine-cli -- new forest
```

#### `choose <index>`
Triggers an interactable choice, action, or link on the active page by its zero-based integer index:
```bash
cargo run --bin ifengine-cli -- choose 0
cargo run --bin ifengine-cli -- choose 1
```

#### `view`
Re-renders the current page, status bar, and list of available choices.

#### `action <name> [args...]` *(Aliases: `act`, `do`)*
Applies a state action, cheat, or inspection command to the active game.

##### Universal & Chant Cheats / Actions
| Action | Description | Example |
| :--- | :--- | :--- |
| `inventory [open\|close\|toggle\|list]` | Inspect or toggle the inventory modal | `action inv toggle` |
| `items` | Print inventory pockets and worn gear | `action items` |
| `money <amount>` | Set currency / guineas | `action money 50` |
| `lives <count>` | Set remaining lives (1..9); auto-activates ghost-sight if ≤ 7 | `action lives 6` |
| `give <item_name>` | Add any story item to inventory (fuzzy name match) | `action give dragon blood` |
| `take <item_name>` | Remove an item from inventory | `action take flute` |
| `set <field> <bool>` | Set story flags directly (`true`/`false`) | `action set tacroy_loyalty 10` |

Supported items for `give` / `take`:
- `sixpence`, `toothbrace`, `skull necklace`, `horse flute`, `shell`, `sand`, `hairpad`, `hatpin`
- `peppermints`, `dragon blood`, `marsh moss`, `arabian nights`, `butter cake`, `silver branch`, `cat`, `aether coin`
- `dream salt`, `chalk charm`, `double ledger`, `crank`, `sea glass`, `guild pass`
- `kitten`, `master key`, `ninth life`, `tigerskin coat`, `soul statue`, `diamond chest`, `flamen warrant`, `lobster rope`

Supported boolean state fields for `set`:
- `tacroy_loyalty`, `tacroy_firmed_up`, `tacroy_soul_restored`
- `millie_trust`, `millie_escaped`, `millie_given_book`
- `ralph_loyalty`, `ralph_suspicion`, `syndicate_power`
- `ghost_sight_active`, `silver_suppressed`, `silver_toothbrace_removed`
- `dredge_sabotaged`, `lobster_pot_rigged`, `dragon_distracted`

##### Forest Cheats / Actions
- `action days <n>`: Set number of elapsed days.
- `action miles <n>`: Set miles traveled.
- `action rations <n>`: Set remaining rations.

##### Test Story Cheats / Actions
- `action weather <condition>`: Set weather string (e.g. `clear`, `stormy`).
- `action miles <n>`: Set miles traveled.
- `action rations <n>`: Set remaining rations.

#### `save <path>`
Exports the active game state to a specified file.
```bash
cargo run --bin ifengine-cli -- save saves/act3_checkpoint.json
```

#### `load <path>`
Loads game state from a specified file and sets it as the active session.
```bash
cargo run --bin ifengine-cli -- load saves/act3_checkpoint.json
```

#### `simulate [story] [depth]` *(Alias: `sim`)*
Runs a headless depth-first traversal of story branches using `ifengine`'s built-in simulation engine. Traverses pages, explores choices, tracks tunnel transitions, and checks for terminal states or dead ends.
```bash
# Simulate default story with depth 4
cargo run --bin ifengine-cli -- simulate chant 4

# Simulate forest story with default depth
cargo run --bin ifengine-cli -- simulate forest
```

---

## Machine-Readable Mode (`--json`)

Adding `--json` outputs structured JSON:
```bash
cargo run --bin ifengine-cli -- --json view
```

Sample output:
```json
{
  "story": "chant",
  "page_id": "chant::act1::p_nursery_intro",
  "status": [
    "Lives: 9/9",
    "Guineas: 0",
    "London — Town House",
    "Inventory: 0"
  ],
  "prose": [
    "Act I: The Night Nursery & The Dreams",
    "The View Through the Banisters",
    "..."
  ],
  "interactables": [
    {
      "id": 0,
      "kind": "action",
      "content": "Papa’s face"
    },
    {
      "id": 1,
      "kind": "action",
      "content": "wide silk skirts"
    },
    {
      "id": 2,
      "kind": "link",
      "content": "He went back to the night nursery, where the dreams began."
    }
  ]
}
```

This output makes it simple to integrate `ifengine-cli` into automated test pipelines, Python scripts, or custom UI runners.
