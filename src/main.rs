use anyhow::{anyhow, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::io::Write;
use std::thread;
use std::time::Duration;
use strava_tui::api::client::StravaClient;
use strava_tui::ui::app::{App, View};

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

fn get_config_path() -> String {
    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "strava-tui", "strava-tui") {
        proj_dirs.config_dir().join("config.toml")
            .to_string_lossy()
            .to_string()
    } else {
        "~/.config/strava-tui/config.toml".to_string()
    }
}

fn config_exists() -> bool {
    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "strava-tui", "strava-tui") {
        proj_dirs.config_dir().join("config.toml").exists()
    } else {
        std::path::Path::new("~/.config/strava-tui/config.toml").exists()
    }
}

fn prompt_for_input(prompt: &str) -> Result<String> {
    loop {
        print!("{}: ", prompt);
        io::stdout().flush()?;
        let mut value = String::new();
        io::stdin().read_line(&mut value)?;
        let value = value.trim().to_string();
        
        if value.is_empty() {
            println!("  Error: This field is required. Please enter a value.");
            continue;
        }
        
        return Ok(value);
    }
}

fn prompt_for_credentials() -> Result<(String, String, String)> {
    let config_path = get_config_path();
    
    println!("\n=== Strava TUI Setup ===\n");
    println!("Config will be saved to: {}\n", config_path);
    
    let client_id = prompt_for_input("Client ID")?;
    let client_secret = prompt_for_input("Client Secret")?;
    let refresh_token = prompt_for_input("Refresh Token")?;
    
    println!("\nCredentials saved!\n");
    
    Ok((client_id, client_secret, refresh_token))
}

fn load_more_activities(client: &StravaClient, page: u32, per_page: u32) -> Result<Vec<strava_tui::api::types::Activity>> {
    let activities = client.get_activities(page, per_page)?;
    Ok(activities)
}

fn run_tui(app: &mut App, client: StravaClient) -> Result<()> {
    let mut terminal = setup_terminal()?;
    
    // Get terminal size to determine initial load count
    let size = terminal.size()?;
    // Account for header (3 lines) and footer (3 lines), each activity takes 1 line
    let activities_per_page = (size.height - 6).max(10) as u32;
    
    // Initial load - load enough to fill the screen
    // Always load at least activities_per_page items
    let new_activities = client.get_activities(1, activities_per_page)?;
    app.add_activities(new_activities, activities_per_page);
    
    let mut pending_load: Option<u32> = None;
    let mut loading = false;

    loop {
        terminal.draw(|f| {
            app.render(f);
        });

        // Handle background loading
        if let Some(page) = pending_load.take() {
            match load_more_activities(&client, page, activities_per_page) {
                Ok(new_activities) => {
                    app.add_activities(new_activities, activities_per_page);
                }
                Err(e) => {
                    app.set_load_error();
                    eprintln!("Failed to load more activities: {}", e);
                }
            }
            loading = false;
        }

        // Check if we should load more (but not if already loading)
        if !loading && app.should_load_more() {
            loading = true;
            app.set_loading(true);
            pending_load = Some(app.activity_page() + 1);
        }

        // Use poll to not block indefinitely
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            restore_terminal().unwrap();
                            break;
                        }
                        KeyCode::Char('d') => {
                            app.set_view(View::Dashboard);
                        }
                        KeyCode::Char('a') => {
                            app.set_view(View::Activities);
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.select_next_activity();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.select_prev_activity();
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            if app.current_view() == View::Activities {
                                app.scroll_left();
                            }
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            if app.current_view() == View::Activities {
                                app.scroll_right();
                            }
                        }
                        KeyCode::Enter => {
                            if app.current_view() == View::Activities && app.get_selected_activity().is_some() {
                                app.set_view(View::ActivityDetail);
                            }
                        }
                        KeyCode::Esc => {
                            if app.current_view() == View::ActivityDetail {
                                app.set_view(View::Activities);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    restore_terminal()?;

    let client = if config_exists() {
        StravaClient::new().map_err(|e| anyhow!("Failed to load config: {}", e))?
    } else {
        println!("No config found. Let's set up Strava TUI!\n");
        println!("1. Create an app at https://www.strava.com/settings/api");
        println!("2. Get your Client ID, Client Secret");
        println!("3. Generate a Refresh Token (see Strava API docs)\n");
        
        let (client_id, client_secret, refresh_token) = prompt_for_credentials()?;
        
        StravaClient::from_credentials(client_id, client_secret, refresh_token)?
    };

    println!("Loading athlete data...");
    
    let athlete = client.get_athlete()?;
    let stats = client.get_athlete_stats(athlete.id)?;
    
    // Activities will be loaded in run_tui() based on terminal size
    let activities = vec![];
    let per_page = 30; // Will be recalculated in run_tui

    let mut app = App::new();
    app.set_data(athlete, stats, activities, per_page);

    if let Err(e) = run_tui(&mut app, client) {
        let _ = restore_terminal();
        eprintln!("Error: {}", e);
    }

    Ok(())
}
