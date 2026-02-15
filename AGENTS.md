# AGENTS.md - Instructions for AI Agents

## Project Overview

- **Name**: SportFrei
- **Type**: Terminal UI application (Rust)
- **Stack**: Rust, ratatui, crossterm, reqwest
- **Tests**: 19 tests in `tests/ui_test.rs`, `tests/types_test.rs`, `tests/api_test.rs`

## Running Commands

```bash
# Run the app
cargo run

# Run tests
cargo test

# Build
cargo build

# Lint (run before committing)
cargo clippy -- -D warnings

# Format
cargo fmt
```

## Pre-commit Checklist

Before committing:
1. Run `cargo test` - all tests must pass
2. Run `cargo clippy -- -D warnings` - fix any warnings
3. Run `cargo fmt` - always format code before committing

## Key Files

- `src/ui/app.rs` - Main UI logic, views, rendering
- `src/api/client.rs` - Strava API client
- `src/api/types.rs` - Data types (Activity, Athlete, AthleteStats)
- `tests/ui_test.rs` - UI rendering tests

## Testing Approach

Tests use ratatui's `TestBackend` for headless rendering tests. Run with:
```bash
cargo test
```

## Code Style

- Use existing patterns in `src/ui/app.rs`
- Keep helper methods private
- Add tests for new features
