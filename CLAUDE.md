# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**Zero to Hero: Rust** — a self-paced, project-based 30-day curriculum that takes a learner from absolute beginner to confident, production-minded Rust developer. See `CURRICULUM.md` for the full day-by-day plan.

## Common Commands

```bash
cargo check --workspace          # fast compile check across all days
cargo build -p day-NN            # build a specific day
cargo run -p day-NN              # run a specific day's binary
cargo test -p day-NN             # test a specific day
cargo test -p day-NN <name>      # run a single test by name (substring match)
cargo clippy --workspace         # lint everything
cargo fmt --all                  # format everything
```

For day-21 (WebAssembly), use `wasm-pack build day-21` instead of `cargo run`.

## Repository Structure

```
zero2hero-rust/
├── Cargo.toml        # workspace root — all 30 days are members
├── CURRICULUM.md     # full 30-day plan with concepts and projects
└── day-01/ … day-30/ # one Cargo package per day
    ├── Cargo.toml
    └── src/
        ├── main.rs   # binary days (most days)
        └── lib.rs    # library days: 07, 10, 11, 14, 21, 29
```

## Architecture

This is a Cargo workspace — each day is an independent crate that inherits `version` and `edition` from `[workspace.package]`. Days do not depend on each other as crate dependencies; later days that revisit earlier work (e.g. day 24 refactors the day 22–23 notes app) do so by evolving that day's crate in place.

**Library vs binary days:**
- Most days are `[[bin]]` crates with `src/main.rs`
- Days 07, 10, 11, 14, 29 are library crates (`src/lib.rs`) — they produce reusable code, not a runnable binary
- Day 21 (WebAssembly) is `crate-type = ["cdylib", "rlib"]` — built with `wasm-pack`, not `cargo run`

Add dependencies only to that day's `Cargo.toml`, not the workspace root — each day is intentionally self-contained.

## Reference Files

- `CURRICULUM.md` — full 30-day plan with concepts and projects per day
- `PRIMITIVES.md` — comprehensive reference for every Rust primitive type: bit representation, ranges, constants, associated functions, methods, and runnable examples. Covers all integer widths, `f32`/`f64`, `bool`, `char`, `&str`, `String`, `()`, `[T; N]`, and tuples.
