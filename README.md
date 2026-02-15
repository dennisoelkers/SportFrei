# SportFrei

A terminal-based Strava activity viewer built with Rust and ratatui.

## Features

- Browse Strava activities in a terminal UI
- Dashboard with activity stats and trends
- Colorized activity table with duration, pace, heart rate, and more
- Fast and lightweight

## Setup

1. Run `cargo run`
2. Enter your Strava Client Secret when prompted
3. Open the displayed URL in your browser
4. Authorize the application
5. The app will automatically start

The first time you run it, you'll complete OAuth authentication. Subsequent runs will use the saved credentials.

## Controls

- `A` - Activities view
- `D` - Dashboard view
- `Q` - Quit
- `j/k` - Navigate up/down
- `h/l` - Scroll left/right
- `Enter` - View activity details
- `Esc` - Go back

## Development

```bash
# Run
cargo run

# Test
cargo test

# Build
cargo build

# Lint (run before committing)
cargo clippy -- -D warnings

# Format
cargo fmt
```

Before committing, ensure tests pass, clippy is clean, and code is formatted.

## License

MIT
