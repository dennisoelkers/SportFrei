use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Athlete {
    pub id: u64,
    pub username: Option<String>,
    pub firstname: String,
    pub lastname: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub profile: Option<String>,
    pub profile_medium: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityType {
    #[serde(rename = "Ride")]
    pub ride: Option<String>,
    #[serde(rename = "Run")]
    pub run: Option<String>,
    #[serde(rename = "Swim")]
    pub swim: Option<String>,
    #[serde(rename = "Hike")]
    pub hike: Option<String>,
    #[serde(rename = "Walk")]
    pub walk: Option<String>,
    #[serde(rename = "WeightTraining")]
    pub weight_training: Option<String>,
    #[serde(rename = "Yoga")]
    pub yoga: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AthleteStats {
    #[serde(rename = "biggest_ride_distance")]
    pub biggest_ride_distance: Option<f64>,
    #[serde(rename = "biggest_climb_elevation_gain")]
    pub biggest_climb_elevation_gain: Option<f64>,
    pub recent_run_totals: ActivityStats,
    pub recent_ride_totals: ActivityStats,
    pub ytd_run_totals: ActivityStats,
    pub ytd_ride_totals: ActivityStats,
    pub all_run_totals: ActivityStats,
    pub all_ride_totals: ActivityStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStats {
    pub count: u32,
    pub distance: f64,
    pub moving_time: u32,
    pub elapsed_time: u32,
    pub elevation_gain: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub sport_type: String,
    pub start_date: DateTime<Utc>,
    pub start_date_local: DateTime<Utc>,
    pub timezone: String,
    pub distance: f64,
    pub moving_time: u32,
    pub elapsed_time: u32,
    pub total_elevation_gain: f64,
    pub average_speed: Option<f64>,
    pub max_speed: Option<f64>,
    pub average_heartrate: Option<f64>,
    pub max_heartrate: Option<f64>,
    pub calories: Option<f64>,
    pub description: Option<String>,
    pub kudos_count: Option<u32>,
    pub comment_count: Option<u32>,
    pub achievement_count: Option<u32>,
    pub pr_count: Option<u32>,
    pub private: Option<bool>,
    pub commute: Option<bool>,
    pub manual: Option<bool>,
    pub gear_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedActivity {
    #[serde(flatten)]
    pub activity: Activity,
    pub segment_efforts: Option<Vec<SegmentEffort>>,
    pub splits_metric: Option<Vec<Split>>,
    pub splits_standard: Option<Vec<Split>>,
    pub laps: Option<Vec<Lap>>,
    pub best_efforts: Option<Vec<BestEffort>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentEffort {
    pub id: u64,
    pub name: String,
    pub activity: Reference,
    pub athlete: Reference,
    pub elapsed_time: u32,
    pub moving_time: u32,
    pub start_date: DateTime<Utc>,
    pub start_date_local: DateTime<Utc>,
    pub distance: f64,
    pub average_speed: f64,
    pub max_speed: f64,
    pub average_heartrate: Option<f64>,
    pub max_heartrate: Option<f64>,
    pub pr_rank: Option<u32>,
    pub pr_elapsed_time: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Split {
    pub distance: f64,
    pub elapsed_time: u32,
    pub elevation_difference: f64,
    pub moving_time: u32,
    pub split: u32,
    pub pace_zone: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lap {
    pub id: u64,
    pub name: String,
    pub activity: Reference,
    pub athlete: Reference,
    pub elapsed_time: u32,
    pub moving_time: u32,
    pub start_date: DateTime<Utc>,
    pub start_date_local: DateTime<Utc>,
    pub distance: f64,
    pub average_speed: f64,
    pub max_speed: f64,
    pub average_heartrate: Option<f64>,
    pub max_heartrate: Option<f64>,
    pub lap_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestEffort {
    pub id: u64,
    pub name: String,
    pub activity: Reference,
    pub athlete: Reference,
    pub elapsed_time: u32,
    pub moving_time: u32,
    pub start_date: DateTime<Utc>,
    pub start_date_local: DateTime<Utc>,
    pub distance: f64,
    pub pr_rank: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub id: u64,
    pub resource_state: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: Option<String>,
    pub errors: Option<Vec<StravaError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StravaError {
    pub resource: String,
    pub field: String,
    pub code: String,
}
