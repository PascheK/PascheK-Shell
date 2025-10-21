# PascheK Shell — Full Project Documentation (Obsidian)

> Version: v0.1 (Core REPL + Internal Commands + Prompt/Theme + Theme Reload)  
> Maintainer: Killian (PascheK)  
> Language: Rust

---

## 1) What is PascheK Shell?

**PascheK Shell** is a native Rust shell that mixes a classic REPL (Read–Eval–Print–Loop) with a customizable visual identity (prompt + theme) and a modular command system. It executes both **internal commands** (implemented in Rust) and **system binaries** available in your `PATH`.

**Key features (current):**
- Interactive REPL loop
- Internal commands (`hello`, `clear`, `cd`, `help`, `theme reload`)
- Dynamic prompt (current dir + time + styled label/symbols)
- Theme configurable via `config/theme.toml`
- Hot reload of theme via `theme reload`

**Planned features:** autocompletion & history, TUI explorer (ratatui), aliases, and plugin system.

---

## 2) Project Structure

```
paschek-shell/
├─ Cargo.toml
└─ src/
   ├─ main.rs                 # Entry point: starts the REPL
   └─ shell/
      ├─ mod.rs               # Shell root module (re-exports submodules)
      ├─ repl.rs              # REPL loop (read input, render prompt, dispatch execution)
      ├─ executor.rs          # Command execution pipeline (internal first, then system)
      ├─ commands/            # Internal commands
      │  ├─ mod.rs            # Command trait + CommandRegistry
      │  ├─ hello.rs          # `hello` command (demo)
      │  ├─ clear.rs          # `clear` command (ANSI clear screen)
      │  ├─ cd.rs             # `cd` command (change current dir)
      │  ├─ help.rs           # `help` (basic)
      │  └─ theme.rs          # `theme reload` (hot-reload prompt theme)
      ├─ prompt/              # Prompt system
      │  ├─ mod.rs            # Prompt struct (render/reload)
      │  ├─ theme.rs          # Theme struct + from Toml + color helpers
      │  └─ builder.rs        # Build final prompt string
      └─ config.rs            # ThemeConfig + TOML loader
```

---

## 3) Control Flow (High-Level)

1. **`main.rs`** calls `shell::repl::start_repl()`.
2. **REPL**:
   - Creates shared `Prompt` (`Arc<Mutex<Prompt>>`).
   - Builds a `CommandRegistry` with `new_with_prompt(prompt.clone())`.
   - Loop:
     - Renders prompt → `prompt.render()` using `builder::build_prompt(theme)`.
     - Reads input line → trims → exit on `"exit"`.
     - Delegates dispatch → `executor::execute_command(...)`.
3. **Executor**:
   - Parses input into `cmd` and `args`.
   - Tries internal registry → `registry.execute(cmd, args)`.
   - If not found → spawns system process (`std::process::Command`).
   - Prints `stdout`/`stderr` if any.
4. **Theme Reload**:
   - `theme reload` → locks `prompt` → `prompt.reload()` → re-reads `config/theme.toml` → updates colors.

---

## 4) Internal Commands

### 4.1 `hello`
- **Goal:** demo message.
- **Usage:** `hello`
- **Effect:** prints “Hello from PascheK Shell 🦀”.

### 4.2 `clear`
- **Goal:** clear terminal with ANSI escape sequences.
- **Usage:** `clear`

### 4.3 `cd`
- **Goal:** change the current working directory (process-wide).
- **Usage:** `cd <path>`
- **Notes:** affects the current process; relative or absolute paths supported; prints error on failure.

### 4.4 `help`
- **Goal:** show minimal usage hint.
- **Usage:** `help`
- **Notes:** placeholder; can be improved by listing `CommandRegistry` content.

### 4.5 `theme reload`
- **Goal:** hot-reload prompt theme from TOML.
- **Usage:** `theme reload`
- **Notes:** reloads `config/theme.toml` without restarting the shell.

---

## 5) Prompt & Theme System

### 5.1 Prompt Rendering
- `Prompt::render()` → `builder::build_prompt(&theme)` returns a **colored** string like:
  ```
  PascheK> • current_dir 22:45:13 
  ```
- Segments:
  - **Shell label**: `PascheK>`
  - **Symbol** (bullet): `•`
  - **Path**: current directory name
  - **Time**: HH:MM:SS (local)

### 5.2 Theme (`prompt/theme.rs`)
- `Theme` holds colors for each segment using `owo_colors::AnsiColors`.
- `Theme::default()` → sensible bright colors.
- `Theme::from_config(cfg)` → parse `ThemeConfig` (from TOML).
- `apply_*` helpers → colorize individual segments consistently.

### 5.3 TOML Format (`config/theme.toml`)
```toml
[shell]
color = "BrightGreen"

[path]
color = "BrightBlue"

[time]
color = "BrightYellow"

[symbol]
color = "BrightMagenta"
```
Supported names: `Black`, `Red`, `Green`, `Yellow`, `Blue`, `Magenta`, `Cyan`, `White`, `BrightGreen`, `BrightBlue`, `BrightYellow`, `BrightMagenta`, `BrightCyan` (case-insensitive).

---

## 6) Error Handling

- **Command not found (system):** clear error printed if binary doesn’t exist.
- **Config reload:** prints warning if file missing/invalid, keeps previous theme.
- **`cd` errors:** message with underlying `std::io::Error`.

---

## 7) Extensibility Guidelines

### 7.1 Add a New Internal Command
1. Create a new file `src/shell/commands/mycmd.rs`.
2. Implement `Command` for a `MyCmd` struct.
3. Register it inside `commands/mod.rs` (`new()` or `new_with_prompt()` if it needs shared state).

Template:
```rust
use super::Command;

pub struct MyCmd;

impl Command for MyCmd {
    fn name(&self) -> &'static str { "mycmd" }
    fn description(&self) -> &'static str { "Describe behavior." }
    fn execute(&self, args: &[&str]) {
        // your logic
    }
}
```

### 7.2 Customize Prompt Layout
- Edit `prompt/builder.rs` to add segments (e.g., username, hostname, git branch).
- Add fields to `Theme` if you need separate colors.
- Extend `ThemeConfig` and `theme.toml` accordingly.

### 7.3 TUI Integration (Future)
- Introduce `ratatui` + `crossterm`.
- Keep REPL for power users; toggle TUI mode with a command (`ui` / `explorer`).

---

## 8) Build & Run

```bash
# Build
cargo build

# Run
cargo run
```

Inside the shell:
```bash
hello
cd src
clear
theme reload
exit
```

---

## 9) Roadmap

- **Phase 5:** Autocompletion + History (`rustyline`/`reedline`), history file `~/.paschek_history`.
- **Phase 6:** TUI explorer with `ratatui`: filesystem panel, status bar, keymaps.
- **Phase 7:** Aliases + config (`config/config.toml`).
- **Phase 8:** Plugin system (Rust crates or scripting) with a safe interface.
- **Phase 9:** Packaging & Releases.

---

## 10) Glossary

- **REPL**: Read–Eval–Print–Loop (interactive loop reading commands).  
- **Internal Command**: Command implemented in Rust within the shell.  
- **System Command**: External executable resolved from `PATH`.  
- **Theme**: Visual style for prompt segments (colors).  
- **TOML**: Config file format used to store theme and settings.

---

## 11) Developer Notes (Rust Concepts)

- **Modules**: split code across files (`mod.rs` + submodules).  
- **Traits**: like interfaces; `Command` defines a contract for all internal commands.  
- **Ownership & Borrowing**: not heavily exposed in this code; `Arc<Mutex<Prompt>>` is used to share mutable state safely between modules.  
- **Arc/Mutex**: thread-safe shared ownership and interior mutability (here, used simply to pass shared prompt state).  
- **Error Handling**: using `match`, `Result`, and printing messages to keep UX simple.  
- **Crates used**:
  - `chrono`: get current time in `builder.rs`
  - `owo-colors`: colorize text
  - `toml` + `serde`: parse theme configuration

---

## 12) Quick Reference (Cheat Sheet)

- **Start shell** → `cargo run`
- **Internal commands**:
  - `hello` → demo greeting
  - `clear` → clear screen
  - `cd <path>` → change dir
  - `help` → basic help message
  - `theme reload` → reload theme from TOML
- **Prompt layout** → `PascheK> • <cwd> <HH:MM:SS>`

---

**End of document.**
