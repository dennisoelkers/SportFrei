# SportFrei

A terminal-based Strava activity viewer built with Rust and ratatui.

## Features

- Browse Strava activities in a terminal UI
- Dashboard with activity stats and trends
- Colorized activity table with duration, pace, heart rate, and more
- Fast and lightweight

## Setup

1. Create an application at https://www.strava.com/settings/api
2. Get your Client ID, Client Secret
3. Generate a Refresh Token (see Strava API docs)
4. Run `cargo run` and follow the setup prompts

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
```

## License

MIT
