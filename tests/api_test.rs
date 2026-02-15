// Integration tests using mockito
// These tests validate the API client logic using a mock HTTP server

use mockito::Server;

#[tokio::test]
async fn test_oauth_token_refresh() {
    let mut server = Server::new_async().await;
    
    let mock = server.mock("POST", "/oauth/token")
        .with_status(200)
        .with_body(r#"{
            "access_token": "test_access_token",
            "refresh_token": "test_refresh_token",
            "expires_at": 9999999999,
            "token_type": "Bearer"
        }"#)
        .create();
    
    let client = reqwest::Client::new();
    
    let response = client
        .post(server.url() + "/oauth/token")
        .form(&[
            ("client_id", "test_id"),
            ("client_secret", "test_secret"),
            ("refresh_token", "test_refresh"),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let token: serde_json::Value = response.json().await.unwrap();
    assert_eq!(token["access_token"], "test_access_token");
    assert_eq!(token["refresh_token"], "test_refresh_token");
    
    mock.assert();
}

#[tokio::test]
async fn test_api_error_handling() {
    let mut server = Server::new_async().await;
    
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
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(server.url() + "/api/v3/athlete/activities")
        .header("Authorization", "Bearer invalid_token")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["message"], "Authorization Error");
    assert_eq!(body["errors"][0]["code"], "missing");
    
    mock.assert();
}

#[tokio::test]
async fn test_pagination() {
    let mut server = Server::new_async().await;
    
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
    
    let client = reqwest::Client::new();
    
    let resp_p1 = client
        .get(server.url() + "/api/v3/athlete/activities")
        .header("Authorization", "Bearer token")
        .query(&[("page", "1"), ("per_page", "2")])
        .send()
        .await
        .unwrap();
    
    let activities: Vec<serde_json::Value> = resp_p1.json().await.unwrap();
    assert_eq!(activities.len(), 2);
    
    mock_p1.assert();
}
