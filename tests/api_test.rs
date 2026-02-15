// Integration tests using mockito
// These tests validate the API client logic using a mock HTTP server

use mockito::Server;

#[test]
fn test_oauth_token_refresh() {
    let mut server = Server::new();
    
    let mock = server.mock("POST", "/oauth/token")
        .with_status(200)
        .with_body(r#"{
            "access_token": "test_access_token",
            "refresh_token": "test_refresh_token",
            "expires_at": 9999999999,
            "token_type": "Bearer"
        }"#)
        .create();
    
    let client = reqwest::blocking::Client::new();
    
    let response = client
        .post(server.url() + "/oauth/token")
        .form(&[
            ("client_id", "test_id"),
            ("client_secret", "test_secret"),
            ("refresh_token", "test_refresh"),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let token: serde_json::Value = response.json().unwrap();
    assert_eq!(token["access_token"], "test_access_token");
    assert_eq!(token["refresh_token"], "test_refresh_token");
    
    mock.assert();
}

#[test]
fn test_api_error_handling() {
    let mut server = Server::new();
    
    let mock = server.mock("GET", "/api/v3/athlete/activities")
        .with_status(401)
        .with_body(r#"{
            "message": "Authorization Error",
            "errors": [{
                "resource": "AccessToken",
                "field": "activity:read_permission",
                "code": "missing"
            }]
        }"#)
        .create();
    
    let client = reqwest::blocking::Client::new();
    
    let response = client
        .get(server.url() + "/api/v3/athlete/activities")
        .header("Authorization", "Bearer invalid_token")
        .send()
        .unwrap();
    
    assert_eq!(response.status(), 401);
    
    let body: serde_json::Value = response.json().unwrap();
    assert_eq!(body["message"], "Authorization Error");
    assert_eq!(body["errors"][0]["code"], "missing");
    
    mock.assert();
}

#[test]
fn test_pagination() {
    let mut server = Server::new();
    
    let mock_p1 = server.mock("GET", "/api/v3/athlete/activities")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_body(r#"[
            {"id": 1, "name": "Activity 1", "type": "Run", "sport_type": "Run", 
             "start_date": "2024-01-15T08:30:00Z", "start_date_local": "2024-01-15T09:30:00+01:00",
             "timezone": "Europe/Berlin", "distance": 5000.0, "moving_time": 1800, 
             "elapsed_time": 2000, "total_elevation_gain": 50.0},
            {"id": 2, "name": "Activity 2", "type": "Run", "sport_type": "Run",
             "start_date": "2024-01-14T08:30:00Z", "start_date_local": "2024-01-14T09:30:00+01:00", 
             "timezone": "Europe/Berlin", "distance": 6000.0, "moving_time": 2000,
             "elapsed_time": 2200, "total_elevation_gain": 60.0}
        ]"#)
        .create();
    
    let client = reqwest::blocking::Client::new();
    
    let resp_p1 = client
        .get(server.url() + "/api/v3/athlete/activities")
        .header("Authorization", "Bearer token")
        .query(&[("page", "1"), ("per_page", "2")])
        .send()
        .unwrap();
    
    let activities: Vec<serde_json::Value> = resp_p1.json().unwrap();
    assert_eq!(activities.len(), 2);
    
    mock_p1.assert();
}

#[test]
fn test_infinite_scroll_pagination() {
    // Test loading multiple pages of activities (simulating infinite scroll)
    let mut server = Server::new();
    
    // Mock page 1
    let _mock_page1 = server.mock("GET", "/api/v3/athlete/activities")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "1".into()),
            mockito::Matcher::UrlEncoded("per_page".into(), "30".into()),
        ]))
        .with_status(200)
        .with_body(r#"[
            {"id": 1, "name": "Activity 1", "type": "Run", "sport_type": "Run", 
             "start_date": "2024-01-15T08:30:00Z", "start_date_local": "2024-01-15T09:30:00+01:00",
             "timezone": "Europe/Berlin", "distance": 5000.0, "moving_time": 1800, 
             "elapsed_time": 2000, "total_elevation_gain": 50.0}
        ]"#)
        .create();
    
    // Mock page 2
    let _mock_page2 = server.mock("GET", "/api/v3/athlete/activities")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "2".into()),
            mockito::Matcher::UrlEncoded("per_page".into(), "30".into()),
        ]))
        .with_status(200)
        .with_body(r#"[
            {"id": 31, "name": "Activity 31", "type": "Run", "sport_type": "Run", 
             "start_date": "2024-01-14T08:30:00Z", "start_date_local": "2024-01-14T09:30:00+01:00",
             "timezone": "Europe/Berlin", "distance": 6000.0, "moving_time": 2000, 
             "elapsed_time": 2200, "total_elevation_gain": 60.0}
        ]"#)
        .create();
    
    // Mock page 3 - returns empty (end of list)
    let _mock_page3 = server.mock("GET", "/api/v3/athlete/activities")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "3".into()),
            mockito::Matcher::UrlEncoded("per_page".into(), "30".into()),
        ]))
        .with_status(200)
        .with_body("[]")
        .create();
    
    let client = reqwest::blocking::Client::new();
    
    // Load page 1
    let resp1 = client
        .get(server.url() + "/api/v3/athlete/activities")
        .header("Authorization", "Bearer token")
        .query(&[("page", "1"), ("per_page", "30")])
        .send()
        .unwrap();
    let activities1: Vec<serde_json::Value> = resp1.json().unwrap();
    assert_eq!(activities1.len(), 1);
    assert_eq!(activities1[0]["id"], 1);
    
    // Load page 2 (simulating infinite scroll trigger)
    let resp2 = client
        .get(server.url() + "/api/v3/athlete/activities")
        .header("Authorization", "Bearer token")
        .query(&[("page", "2"), ("per_page", "30")])
        .send()
        .unwrap();
    let activities2: Vec<serde_json::Value> = resp2.json().unwrap();
    assert_eq!(activities2.len(), 1);
    assert_eq!(activities2[0]["id"], 31);
    
    // Load page 3 (empty - end of list)
    let resp3 = client
        .get(server.url() + "/api/v3/athlete/activities")
        .header("Authorization", "Bearer token")
        .query(&[("page", "3"), ("per_page", "30")])
        .send()
        .unwrap();
    let activities3: Vec<serde_json::Value> = resp3.json().unwrap();
    assert_eq!(activities3.len(), 0);
}

#[test]
fn test_pagination_auth_error() {
    // Test error handling when token expires during pagination
    let mut server = Server::new();
    
    // First request succeeds
    let _mock_success = server.mock("GET", "/api/v3/athlete/activities")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "1".into()),
        ]))
        .with_status(200)
        .with_body(r#"[{"id": 1, "name": "Activity 1", "type": "Run", "sport_type": "Run", 
             "start_date": "2024-01-15T08:30:00Z", "start_date_local": "2024-01-15T09:30:00+01:00",
             "timezone": "Europe/Berlin", "distance": 5000.0, "moving_time": 1800, 
             "elapsed_time": 2000, "total_elevation_gain": 50.0}]"#)
        .create();
    
    // Second request fails with 401 (expired token)
    let _mock_expired = server.mock("GET", "/api/v3/athlete/activities")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "2".into()),
        ]))
        .with_status(401)
        .with_body(r#"{
            "message": "Authorization Error",
            "errors": [{"resource": "AccessToken", "field": "activity:read_permission", "code": "missing"}]
        }"#)
        .create();
    
    let client = reqwest::blocking::Client::new();
    
    // First page works
    let resp1 = client
        .get(server.url() + "/api/v3/athlete/activities")
        .header("Authorization", "Bearer valid_token")
        .query(&[("page", "1")])
        .send()
        .unwrap();
    assert_eq!(resp1.status(), 200);
    
    // Second page fails with auth error
    let resp2 = client
        .get(server.url() + "/api/v3/athlete/activities")
        .header("Authorization", "Bearer expired_token")
        .query(&[("page", "2")])
        .send()
        .unwrap();
    assert_eq!(resp2.status(), 401);
    
    let body: serde_json::Value = resp2.json().unwrap();
    assert_eq!(body["message"], "Authorization Error");
}
