# Day 19: File I/O and Serialization with serde

> **Project:** Config Manager — define a `Config` struct, save and load it in both JSON and TOML formats, and demonstrate a complete round-trip read/modify/write cycle.

## Learning Objectives

By the end of today you will be able to:
- Read entire files into strings and write strings back to disk using `std::fs`.
- Process large files line-by-line with `BufReader` to avoid loading them all into memory.
- Read user input from `io::stdin()` line-by-line and flush `io::stdout()` before prompts.
- Write generic I/O functions using the `Read` and `Write` traits so they work on files, sockets, and stdin/stdout identically.
- Wrap any stream in `BufReader`/`BufWriter` and explain when buffering matters.
- Use `io::Cursor` to drive I/O code in unit tests without touching the filesystem.
- Derive `Serialize` and `Deserialize` on structs and use serde field attributes.
- Convert Rust values to and from JSON using `serde_json`.
- Convert Rust values to and from TOML using the `toml` crate.
- Implement the load/save pattern — deserialize from file, mutate, serialize back.

---

## Concepts

### 1. Reading and Writing Files

> **Docs:** [`std::fs`](https://doc.rust-lang.org/std/fs/) · [`fs::read_to_string`](https://doc.rust-lang.org/std/fs/fn.read_to_string.html) · [`fs::write`](https://doc.rust-lang.org/std/fs/fn.write.html) · [`BufReader`](https://doc.rust-lang.org/std/io/struct.BufReader.html) · [`BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html)

`std::fs` provides simple, whole-file functions. `fs::read_to_string(path)` reads the entire file into a `String`. `fs::write(path, content)` writes a `&[u8]` or `&str` to a file, creating it if it doesn't exist. Both functions return `io::Result<T>` — propagate errors with `?` in functions that return `Result`.

For large files you do not want to load everything at once. Wrap a `File` in a `BufReader` and iterate with `.lines()` from the `BufRead` trait. Each call returns `io::Result<String>` for one line; use `?` inside the iterator chain or a `for` loop.

```rust
use std::fs;
use std::io::{self, BufRead, BufReader};

fn main() -> io::Result<()> {
    // Write a file
    fs::write("hello.txt", "Line 1\nLine 2\nLine 3\n")?;

    // Read the whole file
    let content = fs::read_to_string("hello.txt")?;
    println!("Whole file:\n{content}");

    // Read line-by-line without loading everything into memory
    let file = fs::File::open("hello.txt")?;
    let reader = BufReader::new(file);

    println!("Line by line:");
    for (i, line) in reader.lines().enumerate() {
        println!("  {i}: {}", line?);
    }

    // Clean up
    fs::remove_file("hello.txt")?;
    Ok(())
}
```

**Expected output:**
```
Whole file:
Line 1
Line 2
Line 3

Line by line:
  0: Line 1
  1: Line 2
  2: Line 3
```

#### Exercise 1.1 — Word Counter

**Goal:** Write a file with several lines, then read it back with `BufReader` and count the total number of words.

Write `"the quick brown fox\njumps over the lazy dog\n"` to `words.txt`. Read it back line by line, split each line on whitespace, and accumulate a word count. Print the total. Remove the file afterwards.

```rust
use std::fs;
use std::io::{BufRead, BufReader};

fn main() -> std::io::Result<()> {
    fs::write("words.txt", "the quick brown fox\njumps over the lazy dog\n")?;

    let file = fs::File::open("words.txt")?;
    let reader = BufReader::new(file);

    let mut word_count = 0usize;
    for line in reader.lines() {
        word_count += line?.split_whitespace().count();
    }

    println!("Word count: {word_count}");
    fs::remove_file("words.txt")?;
    Ok(())
}
```

**Expected output:**
```
Word count: 9
```

> **Hint:** `.split_whitespace()` handles multiple spaces and tabs correctly.

---

### 2. `std::io` Streams — stdin, stdout, stderr

> **Docs:** [`std::io`](https://doc.rust-lang.org/std/io/) · [`io::stdin`](https://doc.rust-lang.org/std/io/fn.stdin.html) · [`io::stdout`](https://doc.rust-lang.org/std/io/fn.stdout.html) · [`io::stderr`](https://doc.rust-lang.org/std/io/fn.stderr.html) · [`Read` trait](https://doc.rust-lang.org/std/io/trait.Read.html) · [`Write` trait](https://doc.rust-lang.org/std/io/trait.Write.html) · [`io::Cursor`](https://doc.rust-lang.org/std/io/struct.Cursor.html) · [`BufWriter`](https://doc.rust-lang.org/std/io/struct.BufWriter.html)

Rust's I/O model is built around two core traits in `std::io`:

- **`Read`** — anything you can pull bytes from: files, sockets, stdin, in-memory buffers
- **`Write`** — anything you can push bytes into: files, sockets, stdout, stderr, in-memory buffers

Every I/O primitive in the standard library implements one or both of these traits. Code that is generic over `R: Read` works identically whether `R` is a `File`, `TcpStream`, or the terminal.

#### stdin — reading user input

`io::stdin()` returns a handle to the process's standard input. Call `.read_line(&mut buf)` to read one line (including the trailing newline) into a `String`. Always call `.trim()` on the result to strip the trailing `\n` (or `\r\n` on Windows).

```rust
use std::io::{self, BufRead};

fn main() {
    print!("Enter your name: ");
    // Flush stdout — print! does not add a newline so the buffer may not flush automatically
    use std::io::Write;
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).expect("failed to read line");

    println!("Hello, {}!", line.trim());
}
```

To read *all* lines until EOF (e.g. when stdin is piped from a file):

```rust
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.expect("I/O error");
        println!("Read: {}", line);
    }
}
```

#### stdout and stderr — explicit writes

`println!` and `eprintln!` are the idiomatic shortcuts. Use `io::stdout()` and `io::stderr()` directly when you need:
- **Flushing** — `stdout` is line-buffered in a terminal but block-buffered when piped; explicit `flush()` ensures output appears immediately
- **`write!` / `writeln!`** — the same as `print!` / `println!` but work on any `Write` implementor, enabling generic code

```rust
use std::io::{self, Write};

fn main() {
    // Explicit stdout write
    let mut out = io::stdout();
    write!(out, "Progress: ").unwrap();
    out.flush().unwrap(); // ensure the partial line appears before sleeping

    writeln!(out, "done!").unwrap();

    // Stderr — for errors and diagnostics, not program output
    let mut err = io::stderr();
    writeln!(err, "Warning: something looks off").unwrap();
}
```

**When to use `eprintln!` vs `writeln!(io::stderr(), ...)`:** `eprintln!` is identical to the latter but shorter. Use `writeln!(io::stderr(), ...)` only when you need to pass stderr around as a `Write` value (e.g. into a function).

#### The `Read` and `Write` traits

Because `File`, `TcpStream`, and `io::Stdin` all implement `Read`, you can write functions that work on any of them:

```rust
use std::io::{self, Read, Write};

fn copy_uppercase<R: Read, W: Write>(mut source: R, mut dest: W) -> io::Result<()> {
    let mut buf = Vec::new();
    source.read_to_end(&mut buf)?;
    let upper: Vec<u8> = buf.iter().map(|b| b.to_ascii_uppercase()).collect();
    dest.write_all(&upper)?;
    Ok(())
}

fn main() -> io::Result<()> {
    // Works on stdin → stdout
    let stdin = io::stdin();
    let stdout = io::stdout();
    copy_uppercase(stdin.lock(), stdout.lock())?;
    Ok(())
}
```

Key `Read` methods:
| Method | Description |
|--------|-------------|
| `read(&mut buf)` | Read up to `buf.len()` bytes; returns how many were read |
| `read_to_end(&mut vec)` | Read until EOF into a `Vec<u8>` |
| `read_to_string(&mut s)` | Read until EOF as UTF-8 into a `String` |

Key `Write` methods:
| Method | Description |
|--------|-------------|
| `write(&buf)` | Write up to `buf.len()` bytes; returns how many were written |
| `write_all(&buf)` | Write all bytes, retrying on partial writes |
| `flush()` | Flush any internal buffer to the underlying destination |

#### `BufReader` and `BufWriter` — buffering any stream

Unbuffered reads and writes issue a system call for every byte. `BufReader<R>` wraps any `Read` and adds an internal buffer so multiple bytes are fetched per system call. `BufWriter<W>` wraps any `Write` and batches writes. Both are zero-cost in normal operation — they only matter when you make many small reads or writes.

```rust
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;

fn handle_connection(stream: TcpStream) -> io::Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    let mut request = String::new();
    reader.read_line(&mut request)?;           // reads efficiently from socket

    writeln!(writer, "HTTP/1.1 200 OK")?;
    writer.flush()?;                           // flushes the write buffer
    Ok(())
}
```

`BufReader` also adds the `BufRead` trait, which provides:
- `.read_line(&mut String)` — read one line, including the `\n`
- `.lines()` — iterator of `io::Result<String>`, one per line

#### `io::Cursor` — in-memory streams

`io::Cursor<T>` wraps any `T: AsRef<[u8]>` and makes it behave like a stream. Use it to test I/O code without touching the filesystem or network:

```rust
use std::io::{self, BufRead, Cursor, Write};

fn process<R: BufRead, W: Write>(mut input: R, mut output: W) -> io::Result<()> {
    for line in input.lines() {
        writeln!(output, ">> {}", line?.to_uppercase())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        let input = Cursor::new("hello\nworld\n");
        let mut output = Vec::new();
        process(input, &mut output).unwrap();
        assert_eq!(output, b">> HELLO\n>> WORLD\n");
    }
}

fn main() -> io::Result<()> {
    process(io::stdin().lock(), io::stdout().lock())
}
```

This pattern — accepting `R: BufRead` and `W: Write` — is how production Rust CLI tools make their I/O logic 100% unit-testable.

#### Exercise 2.1 — Interactive Calculator

**Goal:** Make the Day 02 CLI calculator interactive by reading operands from stdin instead of hardcoding them.

```rust
use std::io::{self, BufRead, Write};

fn read_f64(prompt: &str) -> f64 {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    line.trim().parse().expect("expected a number")
}

fn main() {
    let a = read_f64("Enter first number: ");
    let b = read_f64("Enter second number: ");
    print!("Enter operator (+, -, *, /): ");
    io::stdout().flush().unwrap();
    let mut op_line = String::new();
    io::stdin().lock().read_line(&mut op_line).unwrap();
    let op = op_line.trim().chars().next().expect("no operator");

    match op {
        '+' => println!("{} + {} = {}", a, b, a + b),
        '-' => println!("{} - {} = {}", a, b, a - b),
        '*' => println!("{} * {} = {}", a, b, a * b),
        '/' if b == 0.0 => eprintln!("Error: division by zero"),
        '/' => println!("{} / {} = {}", a, b, a / b),
        _   => eprintln!("Unknown operator: {}", op),
    }
}
```

**Expected interaction:**
```
Enter first number: 12
Enter second number: 4
Enter operator (+, -, *, /): /
12 / 4 = 3
```

> **Hint:** The `eprintln!` calls write to stderr — they will not appear in the program's stdout. Try piping stdout to a file (`cargo run > out.txt`) and notice that only the error lines appear in the terminal while the result goes to the file.

#### Exercise 2.2 — Line Counter with `Cursor` Tests

**Goal:** Write a generic `count_lines<R: BufRead>(r: R) -> usize` and test it with `Cursor`.

```rust
use std::io::{BufRead, Cursor};

fn count_lines<R: BufRead>(reader: R) -> usize {
    reader.lines().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        assert_eq!(count_lines(Cursor::new("")), 0);
    }

    #[test]
    fn three_lines() {
        assert_eq!(count_lines(Cursor::new("a\nb\nc\n")), 3);
    }

    #[test]
    fn no_trailing_newline() {
        assert_eq!(count_lines(Cursor::new("a\nb")), 2);
    }
}

fn main() {
    let n = count_lines(std::io::stdin().lock());
    println!("Lines: {}", n);
}
```

Run the tests with `cargo test -p day-19`. Pass input via stdin with `echo -e "a\nb\nc" | cargo run -p day-19`.

> **Hint:** `.lines()` splits on `\n` and strips the newline from each line. A string with no trailing `\n` still counts its last "line."

---

### 3. serde Basics — Derive Macros and Attributes

> **Docs:** [serde.rs](https://serde.rs/) · [serde — Derive](https://serde.rs/derive.html) · [serde — Attributes](https://serde.rs/attributes.html)

`serde` is a *serialization framework*: it defines the `Serialize` and `Deserialize` traits but does not perform actual encoding. Format-specific crates (`serde_json`, `toml`, `serde_yaml`, `bincode`) implement the actual encoding by implementing serde's `Serializer`/`Deserializer` traits.

Deriving both traits on a struct is usually a one-line change: `#[derive(Serialize, Deserialize)]`. The derive macros inspect your struct at compile time and generate the trait implementations. Field attributes let you customise the generated code:

- `#[serde(rename = "other_name")]` — use a different key name in the serialized output.
- `#[serde(default)]` — fill in the field with `Default::default()` if it is absent when deserializing.
- `#[serde(skip)]` — exclude the field entirely.
- `#[serde(skip_serializing_if = "Option::is_none")]` — omit `None` fields from JSON output.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    #[serde(rename = "full_name")]
    name: String,
    #[serde(default)]
    active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
}

fn main() {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        active: true,
        email: None,
    };

    // Serialize to JSON with serde_json (shown in next section)
    println!("User: {:?}", user);
    println!("active default would be: {}", bool::default());
}
```

**Expected output:**
```
User: User { id: 1, name: "Alice", active: true, email: None }
active default would be: false
```

#### Exercise 2.1 — Attribute Practice

**Goal:** Define a `Product` struct with serde attributes. Rename `product_name` to `"name"` in JSON, give `in_stock` a default of `true`, and skip `internal_notes` entirely.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Product {
    id: u32,
    #[serde(rename = "name")]
    product_name: String,
    price: f64,
    #[serde(default = "default_in_stock")]
    in_stock: bool,
    #[serde(skip)]
    internal_notes: String,
}

fn default_in_stock() -> bool { true }
```

**Expected output:** (no runtime output — this is a compilation + attribute exercise)
```
Product compiles with serde attributes.
```

> **Hint:** `#[serde(default = "fn_name")]` calls a function with signature `fn() -> T` when the field is absent.

---

### 4. JSON with `serde_json`

> **Docs:** [`serde_json`](https://docs.rs/serde_json/latest/serde_json/) · [serde_json — Value](https://docs.rs/serde_json/latest/serde_json/enum.Value.html)

`serde_json` is the most widely-used serde format crate. Key functions:

- `serde_json::to_string(&val)` — compact JSON string.
- `serde_json::to_string_pretty(&val)` — indented, human-readable JSON.
- `serde_json::from_str::<T>(s)` — parse a JSON string into `T`.
- `serde_json::from_reader(reader)` — parse from any `io::Read` (e.g., a file).
- `serde_json::Value` — dynamic JSON (when you do not know the schema at compile time).

All functions return `serde_json::Result<T>` — use `?` in functions returning `Result`.

```rust
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct Point {
    x: f64,
    y: f64,
    label: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let point = Point { x: 3.14, y: 2.71, label: "origin".to_string() };

    // Serialize
    let compact = serde_json::to_string(&point)?;
    let pretty = serde_json::to_string_pretty(&point)?;
    println!("Compact: {compact}");
    println!("Pretty:\n{pretty}");

    // Deserialize
    let json = r#"{"x": 1.0, "y": 2.0, "label": "other"}"#;
    let parsed: Point = serde_json::from_str(json)?;
    println!("Parsed: {:?}", parsed);

    // Dynamic JSON value
    let v: serde_json::Value = serde_json::from_str(json)?;
    println!("label field: {}", v["label"]);

    Ok(())
}
```

**Expected output:**
```
Compact: {"x":3.14,"y":2.71,"label":"origin"}
Pretty:
{
  "x": 3.14,
  "y": 2.71,
  "label": "origin"
}
Parsed: Point { x: 1.0, y: 2.0, label: "other" }
label field: "other"
```

#### Exercise 3.1 — JSON Round-Trip

**Goal:** Define a `ServerConfig` struct with fields `host: String`, `port: u16`, `workers: u32`. Serialize it to a JSON string, then deserialize it back into a new `ServerConfig`. Assert that the fields match.

**Expected output:**
```
JSON: {"host":"localhost","port":8080,"workers":4}
Deserialized: ServerConfig { host: "localhost", port: 8080, workers: 4 }
Round-trip OK: true
```

> **Hint:** Compare individual fields — `ServerConfig` does not derive `PartialEq` by default, so add that derive.

---

### 5. TOML with the `toml` Crate

> **Docs:** [`toml` crate](https://docs.rs/toml/latest/toml/) · [TOML spec](https://toml.io/en/)

TOML (Tom's Obvious Minimal Language) is a configuration file format designed to be human-writable. Cargo uses it for `Cargo.toml`. The `toml` crate integrates with serde through the same `Serialize`/`Deserialize` derives.

- `toml::to_string(&val)` — serialise to TOML text.
- `toml::from_str::<T>(s)` — deserialise from TOML text.

TOML has a few restrictions compared to JSON: keys must be strings, arrays must be homogeneous, and nested tables use `[section]` syntax. These translate naturally from nested Rust structs.

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Database {
    host: String,
    port: u16,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    version: String,
    debug: bool,
    database: Database,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig {
        version: "1.0.0".to_string(),
        debug: false,
        database: Database {
            host: "db.internal".to_string(),
            port: 5432,
            name: "production".to_string(),
        },
    };

    let toml_str = toml::to_string(&config)?;
    println!("TOML:\n{toml_str}");

    let parsed: AppConfig = toml::from_str(&toml_str)?;
    println!("Parsed version: {}", parsed.version);
    println!("DB host: {}", parsed.database.host);

    Ok(())
}
```

**Expected output:**
```
TOML:
version = "1.0.0"
debug = false

[database]
host = "db.internal"
port = 5432
name = "production"

Parsed version: 1.0.0
DB host: db.internal
```

#### Exercise 4.1 — TOML Round-Trip

**Goal:** Create a `FeatureFlags` struct with three `bool` fields. Serialise to TOML, write to `flags.toml`, read it back with `fs::read_to_string`, deserialise, and assert all fields match. Clean up the file.

**Expected output:**
```
TOML written to flags.toml
Read back: FeatureFlags { dark_mode: true, beta_features: false, analytics: true }
Round-trip OK: true
```

> **Hint:** The write-to-file step is `fs::write("flags.toml", toml_str)?;` — `toml::to_string` returns a `String`.

---

### 6. Combining File I/O and serde

> **Docs:** [`std::path::Path`](https://doc.rust-lang.org/std/path/struct.Path.html) · [`std::path::PathBuf`](https://doc.rust-lang.org/std/path/struct.PathBuf.html)

The standard load/save pattern:

1. `fs::read_to_string(path)?` — read raw text.
2. `serde_json::from_str(&text)?` (or `toml::from_str`) — deserialise.
3. Mutate the in-memory struct.
4. `serde_json::to_string_pretty(&val)?` — serialise.
5. `fs::write(path, serialised)?` — persist.

Wrapping this in `impl Config` methods keeps your business logic clean.

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Default)]
struct Prefs {
    theme: String,
    font_size: u32,
    auto_save: bool,
}

impl Prefs {
    fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let text = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text)?)
    }

    fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let text = serde_json::to_string_pretty(self)?;
        fs::write(path, text)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "prefs.json";

    // Create and save
    let mut prefs = Prefs { theme: "dark".to_string(), font_size: 14, auto_save: true };
    prefs.save(path)?;

    // Load, modify, re-save
    let mut loaded = Prefs::load(path)?;
    println!("Loaded: {:?}", loaded);
    loaded.font_size = 16;
    loaded.save(path)?;

    // Confirm the change persisted
    let confirmed = Prefs::load(path)?;
    println!("Confirmed font_size: {}", confirmed.font_size);
    fs::remove_file(path)?;
    Ok(())
}
```

**Expected output:**
```
Loaded: Prefs { theme: "dark", font_size: 14, auto_save: true }
Confirmed font_size: 16
```

#### Exercise 5.1 — Append to JSON Array

**Goal:** Implement a function `append_entry(path: &str, entry: &str) -> Result<()>` that loads a `Vec<String>` from a JSON file (or starts with an empty vec if the file doesn't exist), pushes `entry`, and saves it back.

```rust
fn append_entry(path: &str, entry: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries: Vec<String> = if std::path::Path::new(path).exists() {
        serde_json::from_str(&std::fs::read_to_string(path)?)?
    } else {
        Vec::new()
    };
    entries.push(entry.to_string());
    std::fs::write(path, serde_json::to_string_pretty(&entries)?)?;
    Ok(())
}
```

Call it 3 times with different entries. Load and print the final list. Clean up.

**Expected output:**
```
["first", "second", "third"]
```

> **Hint:** `Path::new(path).exists()` checks for the file before trying to read it.

---

## Day Project: Config Manager

### What You're Building

You will implement a `Config` struct that supports loading and saving in both JSON and TOML. The project demonstrates the full serde workflow: derive macros, format-specific crates, file I/O, and round-trip correctness. You will create a default config, save it in both formats, load both back, modify a field, and save again — proving the pipeline works end-to-end.

### Requirements

1. Define:
   ```rust
   #[derive(Debug, Serialize, Deserialize, PartialEq)]
   struct Config {
       app_name: String,
       version: String,
       debug: bool,
       max_connections: u32,
       tags: Vec<String>,
   }
   ```
2. Implement `Config::default() -> Self` returning sensible values.
3. Implement `Config::load_json(path: &str) -> Result<Self, Box<dyn Error>>`.
4. Implement `Config::save_json(&self, path: &str) -> Result<(), Box<dyn Error>>`.
5. Implement `Config::load_toml(path: &str) -> Result<Self, Box<dyn Error>>`.
6. Implement `Config::save_toml(&self, path: &str) -> Result<(), Box<dyn Error>>`.
7. In `main`:
   - Create a default config and save to `config.json` and `config.toml`.
   - Load both files back and print them.
   - Assert `json_loaded == toml_loaded`.
   - Modify `debug` to `true` and `max_connections` to `100`.
   - Save the modified config to both files.
   - Load both again and assert the modifications persisted.
   - Print `"All round-trip checks passed."`.
   - Clean up the files with `fs::remove_file`.

### Getting Started

```bash
cargo new day-19
cd day-19
```

Add to `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
```

### Running Your Solution

```bash
cargo run -p day-19
```

Successful output:

```
=== Saving config ===
Saved config.json
Saved config.toml

=== Loading config ===
JSON: Config { app_name: "MyApp", version: "1.0.0", debug: false, max_connections: 10, tags: ["web", "api"] }
TOML: Config { app_name: "MyApp", version: "1.0.0", debug: false, max_connections: 10, tags: ["web", "api"] }
JSON == TOML: true

=== Modifying and re-saving ===
Modified debug=true, max_connections=100
Saved config.json
Saved config.toml
Loaded JSON debug: true
Loaded TOML max_connections: 100

All round-trip checks passed.
```

### Extension Challenges

- **Easy:** Add a `#[serde(rename = "app-name")]` attribute and verify the JSON output uses the hyphenated key.
- **Medium:** Add a `#[serde(skip)]` field `internal_id: u32` that is assigned after loading. Verify it is absent from the serialized output.
- **Hard:** Support a third format: YAML (add the `serde_yaml` crate). Implement `load_yaml` and `save_yaml`. Add the same round-trip test. Note any differences in how YAML represents lists compared to TOML.
