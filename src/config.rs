use crate::SessionError;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub login_url: String,
    pub username: String,
    pub password: String,
    pub webdriver_url: String,
    pub chromedriver_path: Option<String>,
    pub headless: bool,
    pub post_login_path: String,
    pub initial_wait: u64,
    pub wait_increment: u64,
    pub webdriver_args: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, SessionError> {
        Ok(Self {
            login_url: env::var("LOGIN_URL")
                .map_err(|_| SessionError::EnvVarMissing("LOGIN_URL".to_string()))?,
            username: env::var("USER")
                .map_err(|_| SessionError::EnvVarMissing("USER".to_string()))?,
            password: env::var("PASSWORD")
                .map_err(|_| SessionError::EnvVarMissing("PASSWORD".to_string()))?,
            webdriver_url: env::var("WEBDRIVER_URL")
                .unwrap_or_else(|_| "http://localhost:9515".to_string()),
            chromedriver_path: env::var("CHROMEDRIVER_PATH").ok(),
            headless: env::var("HEADLESS")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            post_login_path: env::var("POST_LOGIN_PATH")
                .unwrap_or_else(|_| "Dashboard.aspx".to_string()),
            initial_wait: env::var("INITIAL_WAIT")
                .map(|v| v.parse().unwrap_or(300))
                .unwrap_or(300),
            wait_increment: env::var("WAIT_INCREMENT")
                .map(|v| v.parse().unwrap_or(300))
                .unwrap_or(300),
            webdriver_args: env::var("WEBDRIVER_ARGS")
                .map(|v| v.split(',').map(String::from).collect())
                .unwrap_or_default(),
        })
    }
}