use crate::api::types::{Activity, Athlete, AthleteStats, DetailedActivity, TokenResponse};
use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use parking_lot::Mutex;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Config {
    client_id: String,
    client_secret: String,
    refresh_token: String,
}

pub struct StravaClient {
    client: Client,
    config: Config,
    access_token: Mutex<Option<String>>,
    config_path: PathBuf,
}

impl Clone for StravaClient {
    fn clone(&self) -> Self {
        Self {
            client: Client::new(),
            config: self.config.clone(),
            access_token: Mutex::new(None),
            config_path: self.config_path.clone(),
        }
    }
}

impl StravaClient {
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Err(anyhow!("No config file found"));
        }

        let config_content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&config_content)
            .map_err(|e| anyhow!("Failed to parse config: {}", e))?;

        Ok(Self {
            client: Client::new(),
            config,
            access_token: Mutex::new(None),
            config_path,
        })
    }

    pub fn from_credentials(
        client_id: String,
        client_secret: String,
        refresh_token: String,
    ) -> Result<Self> {
        let config_path = Self::get_config_path()?;

        let config = Config {
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
            refresh_token: refresh_token.clone(),
        };

        let config_content = toml::to_string(&config)?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&config_path, config_content)?;

        Ok(Self {
            client: Client::new(),
            config,
            access_token: Mutex::new(None),
            config_path,
        })
    }

    fn get_config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "strava-tui", "strava-tui")
            .ok_or_else(|| anyhow!("Could not determine config directory"))?;
        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    fn get_access_token(&self) -> Result<String> {
        let mut token_guard = self.access_token.lock();

        if let Some(ref token) = *token_guard {
            return Ok(token.clone());
        }

        let response = self
            .client
            .post("https://www.strava.com/oauth/token")
            .form(&[
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
                ("refresh_token", &self.config.refresh_token),
                ("grant_type", &"refresh_token".to_string()),
            ])
            .send()?
            .json::<TokenResponse>()?;

        *token_guard = Some(response.access_token.clone());
        Ok(response.access_token)
    }

    pub fn get_athlete(&self) -> Result<Athlete> {
        let token = self.get_access_token()?;
        let response = self
            .client
            .get("https://www.strava.com/api/v3/athlete")
            .header("Authorization", format!("Bearer {}", token))
            .send()?
            .json::<Athlete>()?;
        Ok(response)
    }

    pub fn get_athlete_stats(&self, athlete_id: u64) -> Result<AthleteStats> {
        let token = self.get_access_token()?;
        let response = self
            .client
            .get(format!(
                "https://www.strava.com/api/v3/athletes/{}/stats",
                athlete_id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()?
            .json::<AthleteStats>()?;
        Ok(response)
    }

    pub fn get_activities(&self, page: u32, per_page: u32) -> Result<Vec<Activity>> {
        let token = self.get_access_token()?;
        let response = self
            .client
            .get("https://www.strava.com/api/v3/athlete/activities")
            .header("Authorization", format!("Bearer {}", token))
            .query(&[
                ("page", page.to_string()),
                ("per_page", per_page.to_string()),
            ])
            .send()?;

        let status = response.status();
        let text = response.text()?;

        if !status.is_success() {
            if text.contains("activity:read_permission") || text.contains("missing") {
                return Err(anyhow!(
                    "API returned {}. This usually means your token lacks activity read permissions.\n\
                    \n\
                    To fix:\n\
                    1. Go to https://www.strava.com/playground\n\
                    2. Click Authorize and ensure you check 'activity:read' or 'activity:read_all' scope\n\
                    3. Get a new refresh token and update your config",
                    status
                ));
            }
            return Err(anyhow!("API error {}: {}", status, text));
        }

        let activities: Vec<Activity> = serde_json::from_str(&text)?;
        Ok(activities)
    }

    pub fn get_activity(&self, activity_id: u64) -> Result<DetailedActivity> {
        let token = self.get_access_token()?;
        let response = self
            .client
            .get(format!(
                "https://www.strava.com/api/v3/activities/{}",
                activity_id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()?
            .json::<DetailedActivity>()?;
        Ok(response)
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

impl Default for StravaClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default StravaClient")
    }
}
