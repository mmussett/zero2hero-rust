# Day 20: CLI Applications with clap

> **Project:** `todo` CLI — a full-featured command-line todo manager with `add`, `list`, `done`, and `remove` subcommands, persisting todos to `todos.json`.

## Learning Objectives

By the end of today you will be able to:
- Build a CLI with `clap`'s derive API using `#[derive(Parser)]`.
- Define positional arguments, `--option value` flags, boolean flags, and `Option<T>` optional arguments.
- Create subcommands with `#[derive(Subcommand)]` and dispatch to handler functions.
- Parse and validate argument types automatically (e.g., `--count 5` -> `u32`).
- Combine clap with serde to persist CLI state to JSON files between invocations.

---

## Concepts

### 1. clap's Derive API

> **Docs:** [`clap`](https://docs.rs/clap/latest/clap/) · [clap derive](https://docs.rs/clap/latest/clap/_derive/index.html) · [Book — CLI chapter](https://doc.rust-lang.org/book/ch12-00-an-io-project.html)

clap provides two APIs: a builder API (fluent method chains) and a derive API (procedural macros on structs). The derive API is more concise and easier to read. Annotate your argument struct with `#[derive(Parser)]` and clap infers the CLI structure from field types and names.

`#[command(author, version, about)]` reads your name, version, and description from `Cargo.toml` automatically. Each field in the struct becomes a CLI argument; the field name (converted to `kebab-case`) becomes the flag name.

```rust
use clap::Parser;

/// A simple greeting tool
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Name to greet
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u32,
}

fn main() {
    let args = Args::parse();
    for _ in 0..args.count {
        println!("Hello, {}!", args.name);
    }
}
```

Running `cargo run -- Alice --count 3` outputs:

```
Hello, Alice!
Hello, Alice!
Hello, Alice!
```

Running `cargo run -- --help` outputs:

```
A simple greeting tool

Usage: day-20 <NAME> [OPTIONS]

Arguments:
  <NAME>  Name to greet

Options:
  -c, --count <COUNT>  Number of times to greet [default: 1]
  -h, --help           Print help
  -V, --version        Print version
```

#### Exercise 1.1 — First CLI

**Goal:** Write a CLI that takes a required `--message` string and an optional `--repeat` count (default 1), and prints the message that many times.

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Print a message N times")]
struct Args {
    #[arg(short, long)]
    message: String,

    #[arg(short, long, default_value_t = 1)]
    repeat: u32,
}

fn main() {
    let args = Args::parse();
    for _ in 0..args.repeat {
        println!("{}", args.message);
    }
}
```

**Expected output** (`cargo run -- --message "Rust rocks" --repeat 3`):
```
Rust rocks
Rust rocks
Rust rocks
```

> **Hint:** `#[arg(short, long)]` generates both `-m`/`--message` and `-r`/`--repeat` automatically from the field names.

---

### 2. Argument Types — Positional, Options, Flags, Optional, Vec

> **Docs:** [`clap`](https://docs.rs/clap/latest/clap/) · [clap derive](https://docs.rs/clap/latest/clap/_derive/index.html) · [`std::env`](https://doc.rust-lang.org/std/env/)

clap maps Rust types directly to argument semantics:

- `String` or any type implementing `FromStr` as a positional arg: required, no flag needed.
- `Option<T>`: optional — absent means `None`.
- `bool` with `#[arg(long)]`: a flag (`--verbose` sets it to `true`, absence means `false`).
- `Vec<T>` with `#[arg(long)]`: repeatable — can be specified multiple times.
- `default_value_t = expr`: default value for an option.

```rust
use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    /// Input file (positional, required)
    input: String,

    /// Output file (optional)
    #[arg(short, long)]
    output: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Tags to attach (repeatable: --tag foo --tag bar)
    #[arg(long)]
    tag: Vec<String>,

    /// Number of workers
    #[arg(short, long, default_value_t = 4)]
    workers: u32,
}

fn main() {
    let cli = Cli::parse();
    println!("input:   {}", cli.input);
    println!("output:  {:?}", cli.output);
    println!("verbose: {}", cli.verbose);
    println!("tags:    {:?}", cli.tag);
    println!("workers: {}", cli.workers);
}
```

**Expected output** (`cargo run -- myfile.txt --output out.txt --verbose --tag web --tag api`):
```
input:   myfile.txt
output:  Some("out.txt")
verbose: true
tags:    ["web", "api"]
workers: 4
```

#### Exercise 2.1 — File Processor CLI Skeleton

**Goal:** Define a CLI struct for a hypothetical file processor: required positional `path`, optional `--format` (default `"text"`), `--dry-run` boolean flag, and `--max-lines` with default `100u32`. Print all parsed values.

**Expected output** (`cargo run -- data.csv --format csv --dry-run`):
```
path: data.csv
format: csv
dry_run: true
max_lines: 100
```

> **Hint:** `--dry-run` maps to a field named `dry_run: bool` — clap converts underscores to hyphens in flag names automatically.

---

### 3. Subcommands

> **Docs:** [`clap`](https://docs.rs/clap/latest/clap/) · [clap derive](https://docs.rs/clap/latest/clap/_derive/index.html)

Subcommands let you build git-style CLIs (`git add`, `git commit`). Define an enum with `#[derive(Subcommand)]`, where each variant is one subcommand and holds its own arguments as fields. Add a field `#[command(subcommand)] action: Commands` to your main `Parser` struct.

```rust
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(about = "A file utility")]
struct Cli {
    /// Global verbose flag
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    action: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Copy a file from source to destination
    Copy {
        source: String,
        destination: String,
    },
    /// Delete a file
    Delete {
        path: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// List files in a directory
    List {
        #[arg(default_value = ".")]
        directory: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.action {
        Commands::Copy { source, destination } => {
            println!("Copying {source} -> {destination}");
        }
        Commands::Delete { path, force } => {
            if force {
                println!("Force deleting {path}");
            } else {
                println!("Deleting {path} (with confirmation)");
            }
        }
        Commands::List { directory } => {
            println!("Listing: {directory}");
        }
    }
}
```

**Expected output** (`cargo run -- copy a.txt b.txt`):
```
Copying a.txt -> b.txt
```

**Expected output** (`cargo run -- delete important.txt --force`):
```
Force deleting important.txt
```

#### Exercise 3.1 — Calculator Subcommands

**Goal:** Build a CLI with subcommands `add <a> <b>`, `sub <a> <b>`, `mul <a> <b>` where `a` and `b` are `f64`. Each subcommand prints the result.

**Expected output** (`cargo run -- mul 3.5 4`):
```
3.5 * 4 = 14
```

> **Hint:** `#[derive(Subcommand)]` on an enum with struct variants gives each variant its own named fields.

---

### 4. Validation with Value Parsers

> **Docs:** [`clap`](https://docs.rs/clap/latest/clap/) · [clap derive](https://docs.rs/clap/latest/clap/_derive/index.html)

clap parses any type that implements `std::str::FromStr`. For custom validation beyond type parsing, use `#[arg(value_parser = my_fn)]` where `my_fn: fn(&str) -> Result<T, E>` for some error type that implements `Into<Box<dyn Error + Send + Sync>>`.

```rust
use clap::Parser;

fn parse_port(s: &str) -> Result<u16, String> {
    let port: u16 = s.parse().map_err(|_| format!("{s:?} is not a valid port number"))?;
    if port < 1024 {
        return Err(format!("port {port} is reserved (must be >= 1024)"));
    }
    Ok(port)
}

#[derive(Parser, Debug)]
struct ServerArgs {
    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to bind to (must be >= 1024)
    #[arg(long, default_value_t = 8080, value_parser = parse_port)]
    port: u16,
}

fn main() {
    let args = ServerArgs::parse();
    println!("Binding to {}:{}", args.host, args.port);
}
```

**Expected output** (`cargo run -- --port 8080`):
```
Binding to 127.0.0.1:8080
```

**Expected output** (`cargo run -- --port 80`):
```
error: invalid value '80' for '--port <PORT>': port 80 is reserved (must be >= 1024)
```

#### Exercise 4.1 — Validated Priority

**Goal:** Write a `parse_priority` function that accepts `"low"`, `"medium"`, or `"high"` and returns a `String`, rejecting anything else with an error message. Use it as a `value_parser` for a `--priority` argument.

**Expected output** (`cargo run -- --priority critical`):
```
error: invalid value 'critical' for '--priority <PRIORITY>': must be low, medium, or high
```

> **Hint:** `match s { "low" | "medium" | "high" => Ok(s.to_string()), _ => Err(...) }` is the simplest implementation.

---

### 5. Output and Exit Codes

> **Docs:** [`std::process`](https://doc.rust-lang.org/std/process/) · [`std::process::exit`](https://doc.rust-lang.org/std/process/fn.exit.html) · [`std::env::args`](https://doc.rust-lang.org/std/env/fn.args.html)

Print data to stdout with `println!`. Print error messages to stderr with `eprintln!`. Exit with a non-zero code using `std::process::exit(1)` — but prefer returning `Result` from `main` and letting Rust format the error automatically.

```rust
use clap::Parser;

#[derive(Parser)]
struct Args {
    file: String,
}

fn main() {
    let args = Args::parse();
    match std::fs::read_to_string(&args.file) {
        Ok(content) => {
            println!("{content}");
        }
        Err(e) => {
            eprintln!("Error reading {}: {e}", args.file);
            std::process::exit(1);
        }
    }
}
```

#### Exercise 5.1 — Graceful Error Handling

**Goal:** Extend the above to print line count to stdout. If the file does not exist, print a clear error to stderr and exit with code 1.

**Expected output** (file exists):
```
Line count: 42
```

**Expected output** (file missing):
```
Error reading missing.txt: No such file or directory (os error 2)
[exits with code 1]
```

> **Hint:** `eprintln!` writes to stderr; the process exit code is visible with `echo $?` (Unix) or `echo %ERRORLEVEL%` (Windows) after running.

---

## Day Project: `todo` CLI

### What You're Building

A command-line todo manager with four subcommands (`add`, `list`, `done`, `remove`) that persists todos to `todos.json` in the current directory. This project combines clap (Day 20) with serde (Day 19) to build a complete, realistic CLI application.

### Requirements

1. Define `struct Todo { id: u32, task: String, done: bool, priority: String }` with serde derives.
2. Implement `load_todos(path: &str) -> Vec<Todo>` (returns empty vec if file missing).
3. Implement `save_todos(path: &str, todos: &[Todo])`.
4. Subcommands:
   - `add <task> [--priority high|medium|low]` — generate the next `id` (max existing + 1), push, save.
   - `list [--filter done|pending|all]` — filter todos and print as a table.
   - `done <id>` — set `done = true` for the matching todo, save.
   - `remove <id>` — remove the matching todo, save.
5. Auto-generate help with `#[command(about = "...")]` on each subcommand.
6. All subcommands print a confirmation message.

### Getting Started

```bash
cargo new day-20
cd day-20
```

Add to `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### Auto-generated Help

```
$ cargo run -- --help
A command-line todo manager

Usage: day-20 <COMMAND>

Commands:
  add     Add a new todo item
  list    List todo items
  done    Mark a todo as done
  remove  Remove a todo item
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Running Your Solution

```bash
cargo run -p day-20
```

Full demo session:

```bash
$ cargo run -- add "Buy groceries" --priority high
Added todo #1: "Buy groceries" [high]

$ cargo run -- add "Write tests"
Added todo #2: "Write tests" [medium]

$ cargo run -- add "Read the docs" --priority low
Added todo #3: "Read the docs" [low]

$ cargo run -- list
ID  Done  Priority  Task
1   [ ]   high      Buy groceries
2   [ ]   medium    Write tests
3   [ ]   low       Read the docs

$ cargo run -- done 2
Marked todo #2 as done.

$ cargo run -- list --filter done
ID  Done  Priority  Task
2   [x]   medium    Write tests

$ cargo run -- list --filter pending
ID  Done  Priority  Task
1   [ ]   high      Buy groceries
3   [ ]   low       Read the docs

$ cargo run -- remove 3
Removed todo #3.

$ cargo run -- list
ID  Done  Priority  Task
1   [ ]   high      Buy groceries
2   [x]   medium    Write tests
```

### Extension Challenges

- **Easy:** Add a `--sort-by priority` option to `list` that sorts todos by priority (high > medium > low).
- **Medium:** Add a `clear` subcommand that removes all done todos. Add a `--dry-run` flag that shows what would be removed without actually removing it.
- **Hard:** Add `edit <id> [--task "new text"] [--priority high|medium|low]` subcommand that updates an existing todo's fields. Only update fields that were explicitly provided (use `Option<String>` for each editable field).
