use anyhow::{anyhow, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::io::Write;
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

async fn load_more_activities(client: &StravaClient, app: &mut App, page: u32) -> Result<()> {
    let new_activities = client.get_activities(page, 30).await?;
    app.add_activities(new_activities);
    Ok(())
}

fn run_tui(app: &mut App, client: StravaClient) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let rt = tokio::runtime::Runtime::new().unwrap();

    loop {
        terminal.draw(|f| {
            app.render(f);
        });

        if app.should_load_more() {
            app.set_loading(true);
            let page = app.activity_page() + 1;
            let client_clone = client.clone();
            
            match rt.block_on(load_more_activities(&client_clone, app, page)) {
                Ok(_) => {}
                Err(e) => {
                    app.set_load_error();
                    eprintln!("Failed to load more activities: {}", e);
                }
            }
        }

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

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
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

    println!("Loading Strava data...");
    
    let athlete = client.get_athlete().await?;
    let stats = client.get_athlete_stats(athlete.id).await?;
    let activities = client.get_activities(1, 30).await?;
    
    println!("Loaded {} activities", activities.len());

    let mut app = App::new();
    app.set_data(athlete, stats, activities);

    if let Err(e) = run_tui(&mut app, client) {
        let _ = restore_terminal();
        eprintln!("Error: {}", e);
    }

    Ok(())
}
