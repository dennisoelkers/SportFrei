use anyhow::{anyhow, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use sportfrei::api::client::StravaClient;
use sportfrei::ui::app::{App, View};

const REDIRECT_URI: &str = "http://localhost:42424";
const OAUTH_URL: &str = "https://www.strava.com/oauth/authorize";
const TOKEN_URL: &str = "https://www.strava.com/oauth/token";

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

fn read_config() -> Result<(Option<String>, Option<String>, Option<String>)> {
    let config_path = get_config_path();
    if !std::path::Path::new(&config_path).exists() {
        return Ok((None, None, None));
    }
    
    let content = std::fs::read_to_string(&config_path)?;
    
    let client_id = if let Some(start) = content.find("client_id = \"") {
        let rest = &content[start + 12..];
        if let Some(end) = rest.find("\"") {
            Some(rest[..end].to_string())
        } else {
            None
        }
    } else {
        None
    };
    
    let client_secret = if let Some(start) = content.find("client_secret = \"") {
        let rest = &content[start + 17..];
        if let Some(end) = rest.find("\"") {
            Some(rest[..end].to_string())
        } else {
            None
        }
    } else {
        None
    };
    
    let refresh_token = if let Some(start) = content.find("refresh_token = \"") {
        let rest = &content[start + 16..];
        if let Some(end) = rest.find("\"") {
            Some(rest[..end].to_string())
        } else {
            None
        }
    } else {
        None
    };
    
    Ok((client_id, client_secret, refresh_token))
}

fn save_config(client_id: &str, client_secret: &str, refresh_token: &str) -> Result<()> {
    let config_path = get_config_path();
    let content = format!(
        "client_id = \"{}\"\nclient_secret = \"{}\"\nrefresh_token = \"{}\"\n",
        client_id, client_secret, refresh_token
    );
    
    if let Some(parent) = std::path::Path::new(&config_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&config_path, content)?;
    Ok(())
}

fn run_oauth_flow() -> Result<StravaClient> {
    let (stored_client_id, stored_client_secret, _) = read_config()?;
    let mut client_id = stored_client_id.unwrap_or_default();
    let mut client_secret = stored_client_secret.unwrap_or_default();
    
    if client_id.is_empty() {
        println!("\n=== SportFrei Setup ===\n");
        println!("No client ID found. Please enter your Strava Client ID:\n");
        client_id = prompt_for_input("Client ID")?;
    }
    
    if client_secret.is_empty() {
        if client_id.is_empty() {
            println!("\n=== SportFrei Setup ===\n");
        }
        println!("No client secret found. Please enter your Strava Client Secret:\n");
        client_secret = prompt_for_input("Client Secret")?;
    }
    
    // Build OAuth URL
    let auth_url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&scope=read,activity:read_all",
        OAUTH_URL, client_id, REDIRECT_URI
    );
    
    println!("=== SportFrei OAuth ===\n");
    println!("Please open the following URL in your browser:\n");
    println!("{}\n", auth_url);
    println!("Then authorize the application.\n");
    println!("Waiting for authorization...\n");
    
    // Start HTTP server to receive the callback
    let listener = TcpListener::bind("127.0.0.1:42424")?;
    listener.set_nonblocking(true)?;
    
    let mut code: Option<String> = None;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(300); // 5 minutes timeout
    
    while code.is_none() && start.elapsed() < timeout {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut reader = BufReader::new(&stream);
                let mut request = String::new();
                
                // Read HTTP request
                while request.len() < 4096 {
                    let mut line = String::new();
                    if reader.read_line(&mut line)? == 0 {
                        break;
                    }
                    request.push_str(&line);
                    if line == "\r\n" || line == "\n" {
                        break;
                    }
                }
                
                // Parse query string from URL
                if let Some(query_start) = request.find("GET /?") {
                    let query_part = &request[query_start + 5..];
                    if let Some(query_end) = query_part.find(" HTTP") {
                        let query = &query_part[..query_end];
                        
                        // Parse code parameter
                        for param in query.split('&') {
                            if param.starts_with("code=") {
                                code = Some(param[5..].to_string());
                                break;
                            }
                        }
                    }
                }
                
                // Send response
                let response = if code.is_some() {
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Authorized!</h1><p>You can close this window and return to the terminal.</p></body></html>"
                } else {
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Error</h1><p>No authorization code received.</p></body></html>"
                };
                
                stream.write_all(response.as_bytes())?;
                stream.flush()?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(anyhow!("Server error: {}", e));
            }
        }
    }
    
    let code = code.ok_or_else(|| anyhow!("Authorization timed out"))?;
    println!("Authorization received! Exchanging for token...\n");
    
    // Exchange code for token
    let client = reqwest::blocking::Client::new();
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("code", code.as_str()),
        ("grant_type", "authorization_code"),
    ];
    
    let response = client
        .post(TOKEN_URL)
        .form(&params)
        .send()?
        .json::<sportfrei::api::types::TokenResponse>()?;
    
    // Save config with refresh token
    save_config(&client_id, &client_secret, &response.refresh_token)?;
    
    println!("Token saved! Starting SportFrei...\n");
    
    StravaClient::new()
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

fn load_more_activities(client: &StravaClient, page: u32, per_page: u32) -> Result<Vec<sportfrei::api::types::Activity>> {
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
        match StravaClient::new() {
            Ok(c) => c,
            Err(_) => {
                println!("Config exists but failed to load. Re-running OAuth flow...\n");
                run_oauth_flow()?
            }
        }
    } else {
        run_oauth_flow()?
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
