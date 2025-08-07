# Repository Guidelines

## Project Structure & Modules
- `src/main.rs`: Entry point; initializes `ui::TerminalUI` and starts the game loop.
- `src/ui.rs`: Terminal rendering and input handling using `crossterm` (raw mode, key events).
- `src/baccarat.rs`: Core Baccarat logic (deck, scoring, draw rules, state).
- `Cargo.toml`: Crate metadata and dependencies (`crossterm`, `rand`, `bytemuck`).
- `target/`: Build artifacts (ignored).
- Tests: None yet. Add unit tests beside code (`#[cfg(test)]`) or integration tests in `tests/`.

## Build, Run, Test
- `cargo build`: Compile in debug mode.
- `cargo run`: Build and launch the terminal app.
- `cargo test`: Run unit/integration tests.
- `cargo fmt --all`: Format code with rustfmt.
- `cargo clippy --all-targets -- -D warnings`: Lint and treat warnings as errors.

## Coding Style & Naming
- Edition: Rust 2024. Use rustfmt defaults (4‑space indents; max width per toolchain).
- Naming: `snake_case` for functions/variables, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for consts (e.g., `HEARTS`).
- Organization: Keep UI concerns in `ui` and game rules in `baccarat`; avoid cross‑module duplication.
- Errors: Prefer `Result<T, E>` for fallible paths; bubble errors instead of `unwrap` outside tests.

## Testing Guidelines
- Unit tests: Place near logic (e.g., scoring, banker draw rules) in `baccarat.rs` under `#[cfg(test)]`.
- Integration: Create `tests/` files calling public APIs; run with `cargo test`.
- Naming: Use descriptive `snake_case`, e.g., `calculates_banker_draw_rules`.
- Coverage: No threshold enforced; aim to cover winner determination, pair detection, and payouts.

## Commit & Pull Request Guidelines
- Commits: Imperative subject (“Add banker draw rules”), optional body explaining rationale. Conventional Commits welcome (e.g., `feat: add banker draw logic`).
- PRs: Include summary, linked issues, and terminal screenshots/gifs for UI changes. Confirm: builds, runs locally, tests pass, `fmt`/`clippy` clean.

## Security & Configuration Tips
- Terminal safety: Always restore terminal on error/panic (current hook in `ui::run`).
- Dependencies: Keep minimal; audit new crates and features.
- I/O: No network or file I/O expected; discuss before introducing.

