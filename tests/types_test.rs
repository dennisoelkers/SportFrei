use strava_tui::api::types::{Activity, Athlete, AthleteStats, ActivityStats};

#[test]
fn test_parse_athlete() {
    let json = r#"{
        "id": 123456,
        "username": "testuser",
        "firstname": "John",
        "lastname": "Doe",
        "city": "Berlin",
        "country": "Germany",
        "profile": "https://example.com/profile.jpg",
        "profile_medium": "https://example.com/profile_med.jpg"
    }"#;

    let athlete: Athlete = serde_json::from_str(json).unwrap();
    
    assert_eq!(athlete.id, 123456);
    assert_eq!(athlete.username, Some("testuser".to_string()));
    assert_eq!(athlete.firstname, "John");
    assert_eq!(athlete.lastname, "Doe");
    assert_eq!(athlete.city, Some("Berlin".to_string()));
    assert_eq!(athlete.country, Some("Germany".to_string()));
}

#[test]
fn test_parse_activity_stats() {
    let json = r#"{
        "count": 42,
        "distance": 350000.5,
        "moving_time": 3600,
        "elapsed_time": 4000,
        "elevation_gain": 1200.0
    }"#;

    let stats: ActivityStats = serde_json::from_str(json).unwrap();
    
    assert_eq!(stats.count, 42);
    assert!((stats.distance - 350000.5).abs() < 0.01);
    assert_eq!(stats.moving_time, 3600);
    assert_eq!(stats.elapsed_time, 4000);
    assert!((stats.elevation_gain - 1200.0).abs() < 0.01);
}

#[test]
fn test_parse_athlete_stats() {
    let json = r#"{
        "biggestRideDistance": 150000.0,
        "biggestClimbElevationGain": 3000.0,
        "recent_run_totals": {
            "count": 10,
            "distance": 50000.0,
            "moving_time": 18000,
            "elapsed_time": 20000,
            "elevation_gain": 500.0
        },
        "recent_ride_totals": {
            "count": 5,
            "distance": 100000.0,
            "moving_time": 14400,
            "elapsed_time": 15000,
            "elevation_gain": 1000.0
        },
        "ytd_run_totals": {
            "count": 50,
            "distance": 250000.0,
            "moving_time": 90000,
            "elapsed_time": 100000,
            "elevation_gain": 2500.0
        },
        "ytd_ride_totals": {
            "count": 25,
            "distance": 500000.0,
            "moving_time": 72000,
            "elapsed_time": 75000,
            "elevation_gain": 5000.0
        },
        "all_run_totals": {
            "count": 100,
            "distance": 500000.0,
            "moving_time": 180000,
            "elapsed_time": 200000,
            "elevation_gain": 5000.0
        },
        "all_ride_totals": {
            "count": 50,
            "distance": 1000000.0,
            "moving_time": 144000,
            "elapsed_time": 150000,
            "elevation_gain": 10000.0
        }
    }"#;

    let stats: AthleteStats = serde_json::from_str(json).unwrap();
    
    assert_eq!(stats.recent_run_totals.count, 10);
    assert_eq!(stats.recent_ride_totals.count, 5);
    assert_eq!(stats.ytd_run_totals.count, 50);
    assert_eq!(stats.ytd_ride_totals.count, 25);
    assert_eq!(stats.all_run_totals.count, 100);
    assert_eq!(stats.all_ride_totals.count, 50);
}

#[test]
fn test_parse_activity() {
    let json = r#"{
        "id": 123456789,
        "name": "Morning Run",
        "type": "Run",
        "sport_type": "Run",
        "start_date": "2024-01-15T08:30:00Z",
        "start_date_local": "2024-01-15T09:30:00+01:00",
        "timezone": "Europe/Berlin",
        "distance": 5000.0,
        "moving_time": 1800,
        "elapsed_time": 2000,
        "total_elevation_gain": 50.0,
        "average_speed": 2.78,
        "max_speed": 3.5,
        "average_heartrate": 150.0,
        "max_heartrate": 175.0,
        "calories": 350.0,
        "description": "Easy morning run",
        "kudos_count": 5,
        "comment_count": 1,
        "achievement_count": 2,
        "pr_count": 1,
        "private": false,
        "commute": false,
        "manual": false,
        "gear_id": "g123"
    }"#;

    let activity: Activity = serde_json::from_str(json).unwrap();
    
    assert_eq!(activity.id, 123456789);
    assert_eq!(activity.name, "Morning Run");
    assert_eq!(activity.activity_type, "Run");
    assert!((activity.distance - 5000.0).abs() < 0.01);
    assert_eq!(activity.moving_time, 1800);
    assert_eq!(activity.pr_count, Some(1));
    assert_eq!(activity.description, Some("Easy morning run".to_string()));
}

#[test]
fn test_parse_activity_list() {
    let json = r#"[
        {
            "id": 1,
            "name": "Run 1",
            "type": "Run",
            "sport_type": "Run",
            "start_date": "2024-01-15T08:30:00Z",
            "start_date_local": "2024-01-15T09:30:00+01:00",
            "timezone": "Europe/Berlin",
            "distance": 5000.0,
            "moving_time": 1800,
            "elapsed_time": 2000,
            "total_elevation_gain": 50.0
        },
        {
            "id": 2,
            "name": "Ride 1",
            "type": "Ride",
            "sport_type": "Ride",
            "start_date": "2024-01-14T08:30:00Z",
            "start_date_local": "2024-01-14T09:30:00+01:00",
            "timezone": "Europe/Berlin",
            "distance": 20000.0,
            "moving_time": 3600,
            "elapsed_time": 4000,
            "total_elevation_gain": 200.0
        }
    ]"#;

    let activities: Vec<Activity> = serde_json::from_str(json).unwrap();
    
    assert_eq!(activities.len(), 2);
    assert_eq!(activities[0].name, "Run 1");
    assert_eq!(activities[1].name, "Ride 1");
    assert_eq!(activities[0].activity_type, "Run");
    assert_eq!(activities[1].activity_type, "Ride");
}

#[test]
fn test_activity_distance_conversion() {
    let json = r#"{
        "id": 1,
        "name": "Test",
        "type": "Run",
        "sport_type": "Run",
        "start_date": "2024-01-15T08:30:00Z",
        "start_date_local": "2024-01-15T09:30:00+01:00",
        "timezone": "Europe/Berlin",
        "distance": 1000.0,
        "moving_time": 300,
        "elapsed_time": 360,
        "total_elevation_gain": 10.0
    }"#;

    let activity: Activity = serde_json::from_str(json).unwrap();
    
    // Distance is in meters, convert to km
    let distance_km = activity.distance / 1000.0;
    assert!((distance_km - 1.0).abs() < 0.01);
    
    // Moving time is in seconds
    let minutes = activity.moving_time / 60;
    assert_eq!(minutes, 5);
}
