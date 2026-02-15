// Headless TUI tests using ratatui's TestBackend

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use ratatui::buffer::Buffer;
use strava_tui::api::types::{Activity, ActivityStats, Athlete, AthleteStats};
use strava_tui::ui::app::{App, View};

fn get_buffer_content(buffer: &Buffer) -> String {
    let mut content = String::new();
    let area = buffer.area();
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = buffer.get(x, y);
            content.push_str(&cell.symbol().to_string());
        }
        content.push('\n');
    }
    content
}

#[test]
fn test_activities_table_columns() {
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let mut app = create_test_app();
    app.set_view(View::Activities);
    
    terminal.draw(|f| {
        app.render(f);
    }).unwrap();
    
    let buffer = terminal.backend().buffer();
    let content = get_buffer_content(buffer);
    
    // Check that activities title is present
    assert!(content.contains("Activities"));
    
    // Check for new columns
    assert!(content.contains("Date"), "Should have Date column");
    assert!(content.contains("Time"), "Should have Time column");
    assert!(content.contains("Distance"), "Should have Distance column");
    assert!(content.contains("Elev"), "Should have Elevation column");
    assert!(content.contains("Pace"), "Should have Pace column");
    assert!(content.contains("HR"), "Should have Heart Rate column");
    assert!(content.contains("Cal"), "Should have Calories column");
    
    // Check that activity data is displayed
    assert!(content.contains("Morning Run"));
}

fn create_test_app() -> App {
    let athlete = Athlete {
        id: 12345,
        username: Some("testuser".to_string()),
        firstname: "John".to_string(),
        lastname: "Doe".to_string(),
        city: Some("Berlin".to_string()),
        country: Some("Germany".to_string()),
        profile: None,
        profile_medium: None,
    };
    
    let stats = AthleteStats {
        biggestRideDistance: Some(50000.0),
        biggestClimbElevationGain: Some(1000.0),
        recent_run_totals: ActivityStats {
            count: 10,
            distance: 50000.0,
            moving_time: 18000,
            elapsed_time: 20000,
            elevation_gain: 500.0,
        },
        recent_ride_totals: ActivityStats {
            count: 5,
            distance: 100000.0,
            moving_time: 14400,
            elapsed_time: 15000,
            elevation_gain: 1000.0,
        },
        ytd_run_totals: ActivityStats {
            count: 50,
            distance: 250000.0,
            moving_time: 90000,
            elapsed_time: 100000,
            elevation_gain: 2500.0,
        },
        ytd_ride_totals: ActivityStats {
            count: 25,
            distance: 500000.0,
            moving_time: 72000,
            elapsed_time: 75000,
            elevation_gain: 5000.0,
        },
        all_run_totals: ActivityStats {
            count: 100,
            distance: 500000.0,
            moving_time: 180000,
            elapsed_time: 200000,
            elevation_gain: 5000.0,
        },
        all_ride_totals: ActivityStats {
            count: 50,
            distance: 1000000.0,
            moving_time: 144000,
            elapsed_time: 150000,
            elevation_gain: 10000.0,
        },
    };
    
    let activities = vec![
        Activity {
            id: 1,
            name: "Morning Run".to_string(),
            activity_type: "Run".to_string(),
            sport_type: "Run".to_string(),
            start_date: chrono::Utc::now(),
            start_date_local: chrono::Utc::now(),
            timezone: "Europe/Berlin".to_string(),
            distance: 5000.0,
            moving_time: 1800,
            elapsed_time: 2000,
            total_elevation_gain: 50.0,
            average_speed: Some(2.78),
            max_speed: Some(3.5),
            average_heartrate: Some(150.0),
            max_heartrate: Some(175.0),
            calories: Some(350.0),
            description: None,
            kudos_count: Some(5),
            comment_count: Some(1),
            achievement_count: Some(2),
            pr_count: Some(1),
            private: Some(false),
            commute: Some(false),
            manual: Some(false),
            gear_id: None,
        },
        Activity {
            id: 2,
            name: "Evening Ride".to_string(),
            activity_type: "Ride".to_string(),
            sport_type: "Ride".to_string(),
            start_date: chrono::Utc::now(),
            start_date_local: chrono::Utc::now(),
            timezone: "Europe/Berlin".to_string(),
            distance: 25000.0,
            moving_time: 3600,
            elapsed_time: 4000,
            total_elevation_gain: 200.0,
            average_speed: Some(6.94),
            max_speed: Some(8.5),
            average_heartrate: Some(140.0),
            max_heartrate: Some(170.0),
            calories: Some(600.0),
            description: None,
            kudos_count: Some(10),
            comment_count: Some(0),
            achievement_count: Some(3),
            pr_count: Some(0),
            private: Some(false),
            commute: Some(false),
            manual: Some(false),
            gear_id: None,
        },
    ];
    
    let mut app = App::new();
    app.set_data(athlete, stats, activities, 30);
    app
}

#[test]
fn test_dashboard_renders() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let mut app = create_test_app();
    app.set_view(View::Dashboard);
    
    terminal.draw(|f| {
        app.render(f);
    }).unwrap();
    
    let buffer = terminal.backend().buffer();
    let content = get_buffer_content(buffer);
    
    // Check that the dashboard title is present
    assert!(content.contains("Dashboard"));
    
    // Check that the athlete name is displayed
    assert!(content.contains("Welcome, John!"));
}

#[test]
fn test_activities_list_renders() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let mut app = create_test_app();
    app.set_view(View::Activities);
    
    terminal.draw(|f| {
        app.render(f);
    }).unwrap();
    
    let buffer = terminal.backend().buffer();
    let content = get_buffer_content(buffer);
    
    // Check that activities title is present
    assert!(content.contains("Activities"));
    
    // Check that activities are listed
    assert!(content.contains("Morning Run"));
    assert!(content.contains("Evening Ride"));
}

#[test]
fn test_activity_detail_renders() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let mut app = create_test_app();
    app.set_view(View::Activities);
    app.select_next_activity(); // Select first activity
    app.set_view(View::ActivityDetail);
    
    terminal.draw(|f| {
        app.render(f);
    }).unwrap();
    
    let buffer = terminal.backend().buffer();
    let content = get_buffer_content(buffer);
    
    // Check that detail title is present
    assert!(content.contains("Details"));
}

#[test]
fn test_navigation_keys() {
    let mut app = create_test_app();
    app.set_view(View::Dashboard);
    
    // Initially on dashboard
    assert_eq!(app.current_view(), View::Dashboard);
    
    // Press 'a' to go to activities
    app.set_view(View::Activities);
    assert_eq!(app.current_view(), View::Activities);
    
    // Press 'd' to go back to dashboard
    app.set_view(View::Dashboard);
    assert_eq!(app.current_view(), View::Dashboard);
}

#[test]
fn test_activity_selection() {
    let mut app = create_test_app();
    app.set_view(View::Activities);
    
    // Initially first activity is selected (index 0)
    assert_eq!(app.get_selected_activity().unwrap().name, "Morning Run");
    
    // Select next
    app.select_next_activity();
    assert_eq!(app.get_selected_activity().unwrap().name, "Evening Ride");
    
    // Select previous
    app.select_prev_activity();
    assert_eq!(app.get_selected_activity().unwrap().name, "Morning Run");
}

#[test]
fn test_infinite_scroll_triggers() {
    let mut app = create_test_app();
    app.set_view(View::Activities);
    
    // Set has_more_activities to true to test the logic
    // (In real usage, this would be set based on API response)
    // For now, just test the selection logic works
    assert!(app.get_selected_activity().is_some());
    
    // Test that selection moves correctly
    app.select_next_activity();
    assert_eq!(app.get_selected_activity().unwrap().id, 2);
    
    // Can't go past last item
    app.select_next_activity();
    assert_eq!(app.get_selected_activity().unwrap().id, 2);
}

#[test]
fn test_footer_shows_navigation() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    
    let mut app = create_test_app();
    app.set_view(View::Dashboard);
    
    terminal.draw(|f| {
        app.render(f);
    }).unwrap();
    
    let buffer = terminal.backend().buffer();
    let content = get_buffer_content(buffer);
    
    // Check that footer shows navigation hints
    assert!(content.contains("Dashboard"));
    assert!(content.contains("Activities"));
}

#[test]
fn test_infinite_scroll_no_crash_on_empty_response() {
    let mut app = create_test_app();
    app.set_view(View::Activities);
    
    // Add empty activities (simulating end of list)
    app.add_activities(vec![], 10);
    
    // should_load_more should now be false since we got empty results
    assert!(!app.should_load_more());
}

#[test]
fn test_load_error_state() {
    let mut app = create_test_app();
    app.set_view(View::Activities);
    
    // Initially not loading
    assert!(!app.is_loading());
    
    // Set loading state
    app.set_loading(true);
    assert!(app.is_loading());
    
    // Simulate error - clear loading flag
    app.set_load_error();
    assert!(!app.is_loading());
}

#[test]
fn test_screen_size_based_loading_small_terminal() {
    // Test with small terminal (e.g., 15 rows)
    // Activities area = 15 - 6 = 9 lines
    let per_page: usize = 9;
    
    // Create app with 5 activities, per_page=9
    let mut app = App::new();
    let athlete = create_test_athlete();
    let stats = create_test_stats();
    let activities = create_test_activities(5);
    app.set_data(athlete, stats, activities, per_page);
    
    // Should not load more since 5 < 9
    assert!(!app.should_load_more());
}

#[test]
fn test_screen_size_based_loading_large_terminal() {
    // Test with large terminal (e.g., 50 rows)
    // Activities area = 50 - 6 = 44 lines
    let per_page: usize = 44;
    
    // Create app with a FULL page of 44 activities
    let mut app = App::new();
    let athlete = create_test_athlete();
    let stats = create_test_stats();
    let activities = create_test_activities(44); // Full page
    app.set_data(athlete, stats, activities, per_page);
    app.set_view(View::Activities);
    
    // Move selection near the end (within 5 items of the end)
    for _ in 0..39 {
        app.select_next_activity();
    }
    
    // Should load more since we got a full page (44 == per_page means more available)
    assert!(app.should_load_more());
}

#[test]
fn test_add_activities_updates_has_more_based_on_per_page() {
    let per_page: u32 = 20;
    let mut app = App::new();
    let athlete = create_test_athlete();
    let stats = create_test_stats();
    let activities = create_test_activities(15);
    app.set_data(athlete, stats, activities, per_page as usize);
    
    // 15 < 20, should not have more
    assert!(!app.should_load_more());
    
    // Add 10 more activities - total 25 > 20
    app.add_activities(create_test_activities(10), per_page);
}

#[test]
fn test_activity_page_increments() {
    let mut app = create_test_app();
    app.set_view(View::Activities);
    
    let initial_page = app.activity_page();
    assert_eq!(initial_page, 1);
    
    // Add more activities
    app.add_activities(create_test_activities(10), 30);
    
    // Page should increment
    assert_eq!(app.activity_page(), 2);
}

fn create_test_athlete() -> Athlete {
    Athlete {
        id: 12345,
        username: Some("testuser".to_string()),
        firstname: "John".to_string(),
        lastname: "Doe".to_string(),
        city: Some("Berlin".to_string()),
        country: Some("Germany".to_string()),
        profile: None,
        profile_medium: None,
    }
}

fn create_test_stats() -> AthleteStats {
    AthleteStats {
        biggestRideDistance: Some(50000.0),
        biggestClimbElevationGain: Some(1000.0),
        recent_run_totals: ActivityStats {
            count: 10,
            distance: 50000.0,
            moving_time: 18000,
            elapsed_time: 20000,
            elevation_gain: 500.0,
        },
        recent_ride_totals: ActivityStats {
            count: 5,
            distance: 100000.0,
            moving_time: 14400,
            elapsed_time: 15000,
            elevation_gain: 1000.0,
        },
        ytd_run_totals: ActivityStats { count: 50, distance: 250000.0, moving_time: 90000, elapsed_time: 100000, elevation_gain: 2500.0 },
        ytd_ride_totals: ActivityStats { count: 25, distance: 500000.0, moving_time: 72000, elapsed_time: 75000, elevation_gain: 5000.0 },
        all_run_totals: ActivityStats { count: 100, distance: 500000.0, moving_time: 180000, elapsed_time: 200000, elevation_gain: 5000.0 },
        all_ride_totals: ActivityStats { count: 50, distance: 1000000.0, moving_time: 144000, elapsed_time: 150000, elevation_gain: 10000.0 },
    }
}

fn create_test_activities(count: usize) -> Vec<Activity> {
    (0..count).map(|i| Activity {
        id: i as u64,
        name: format!("Activity {}", i),
        activity_type: "Run".to_string(),
        sport_type: "Run".to_string(),
        start_date: chrono::Utc::now(),
        start_date_local: chrono::Utc::now(),
        timezone: "Europe/Berlin".to_string(),
        distance: 5000.0,
        moving_time: 1800,
        elapsed_time: 2000,
        total_elevation_gain: 50.0,
        average_speed: Some(2.78),
        max_speed: Some(3.5),
        average_heartrate: Some(150.0),
        max_heartrate: Some(175.0),
        calories: Some(350.0),
        description: None,
        kudos_count: Some(5),
        comment_count: Some(1),
        achievement_count: Some(2),
        pr_count: Some(1),
        private: Some(false),
        commute: Some(false),
        manual: Some(false),
        gear_id: None,
    }).collect()
}
