use crate::SessionError;
use colored::*;
use std::time::Duration;
use tokio;

pub struct ChromeDriver;

impl ChromeDriver {
    pub async fn check_ready(webdriver_url: &str) -> Result<(), SessionError> {
        let max_retries = 5;
        let status_url = format!("{}/status", webdriver_url);
        let client = reqwest::Client::new();

        for i in 1..=max_retries {
            println!(
                "{} Checking if ChromeDriver is ready (attempt {}/{})",
                "[?]".yellow(),
                i,
                max_retries
            );

            match client.get(&status_url).send().await {
                Ok(_) => {
                    println!(
                        "{} ChromeDriver is ready at {}",
                        "[+]".green(),
                        webdriver_url
                    );
                    return Ok(());
                }
                Err(e) => {
                    println!("{} Attempt {} failed: {}", "[!]".red(), i, e);
                    if i < max_retries {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }

        Err(SessionError::ProcessError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "ChromeDriver is not responding at {} after maximum retries",
                webdriver_url
            ),
        )))
    }
}