use crate::{Config, SessionError};
use chrono::Local;
use colored::*;
use fantoccini::wd::Capabilities;
use fantoccini::{Client, ClientBuilder, Locator};
use reqwest::Url;
use std::io::{self, Write};
use std::time::Duration;
use tokio;

pub struct SessionMonitor {
    client: Client,
    config: Config,
    total_wait_time: Duration,
    current_wait_time: Duration,
}

impl SessionMonitor {
    pub async fn new(config: Config) -> Result<Self, SessionError> {
        let mut caps = Capabilities::new();
        let mut chrome_args = if config.headless {
            vec!["--headless"]
        } else {
            vec![]
        };

        chrome_args.extend_from_slice(&[
            "--log-level=3",
            "--silent",
            "--disable-logging",
            "--disable-gpu",
            "--disable-dev-shm-usage",
            "--no-sandbox",
            "--remote-debugging-port=0",
            "--disable-machine-learning",
            "--disable-background-networking",
            "--disable-component-update",
        ]);

        caps.insert(
            "goog:chromeOptions".to_string(),
            serde_json::json!({
                "args": chrome_args
            }),
        );

        let client = ClientBuilder::native()
            .capabilities(caps)
            .connect(&config.webdriver_url)
            .await?;

        let initial_wait = Duration::from_secs(config.initial_wait);

        Ok(Self {
            client,
            config,
            total_wait_time: Duration::from_secs(0),
            current_wait_time: initial_wait,
        })
    }

    pub async fn login(&mut self) -> Result<(), SessionError> {
        println!("{} Initiating login process...", "[?]".yellow());
        self.client.goto(&self.config.login_url).await?;

        let username_field = self
            .client
            .find(Locator::Css("[name='ctl00$ContentPlaceHolder1$Brukernavn']"))
            .await?;
        let password_field = self
            .client
            .find(Locator::Css("[name='ctl00$ContentPlaceHolder1$Passord']"))
            .await?;

        username_field.send_keys(&self.config.username).await?;
        password_field.send_keys(&self.config.password).await?;

        let submit_button = self
            .client
            .find(Locator::Css("input[type='submit']"))
            .await?;
        submit_button.click().await?;

        self.client.wait().for_element(Locator::Css("body")).await?;

        let post_login_url = format!("{}{}", self.config.login_url, self.config.post_login_path);
        self.client.goto(&post_login_url).await?;

        println!("{} Login successful!", "[+]".green());
        Ok(())
    }

    async fn check_session(&mut self) -> Result<bool, SessionError> {
        self.client.refresh().await?;

        let current_url = self.client.current_url().await?;
        let login_url = Url::parse(&self.config.login_url)?;

        if current_url == login_url {
            Err(SessionError::SessionExpired)
        } else {
            Ok(true)
        }
    }

    async fn display_countdown(&self, remaining: u64) -> io::Result<()> {
        print!("\r\x1B[K");
        let minutes = remaining / 60;
        let seconds = remaining % 60;

        let progress_width = 50;
        let progress = ((self.current_wait_time.as_secs() - remaining) as f64
            / self.current_wait_time.as_secs() as f64
            * progress_width as f64) as usize;

        let progress_bar = format!(
            "[{}{}]",
            "=".repeat(progress),
            " ".repeat(progress_width - progress)
        );

        print!(
            "{} Time remaining: {:02}:{:02} {}",
            "[#]".blue(),
            minutes,
            seconds,
            progress_bar
        );

        io::stdout().flush()
    }

    pub async fn run(&mut self) -> Result<(), SessionError> {
        loop {
            print!("\x1B[2J\x1B[1;1H");
            println!(
                "{} Starting new wait cycle of {} minutes...",
                "[?]".yellow(),
                self.current_wait_time.as_secs() / 60
            );

            for remaining in (0..=self.current_wait_time.as_secs()).rev() {
                self.display_countdown(remaining).await?;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            println!();

            self.total_wait_time += self.current_wait_time;
            let current_time = Local::now();

            match self.check_session().await {
                Ok(_) => {
                    println!("\n{} Session Check Successful", "[+]".green());
                    println!(
                        "{} Time: {}",
                        "[#]".blue(),
                        current_time.format("%Y-%m-%d %H:%M:%S")
                    );
                    println!(
                        "{} Last wait duration: {} minutes",
                        "[#]".blue(),
                        self.current_wait_time.as_secs() / 60
                    );
                    println!(
                        "{} Total time monitored: {} minutes",
                        "[#]".blue(),
                        self.total_wait_time.as_secs() / 60
                    );
                    println!(
                        "\n{} Increasing wait time for next cycle...",
                        "[?]".yellow()
                    );

                    self.current_wait_time += Duration::from_secs(self.config.wait_increment);
                }
                Err(SessionError::SessionExpired) => {
                    println!(
                        "\n{} Session expired after waiting for {} minutes",
                        "[!]".red(),
                        self.current_wait_time.as_secs() / 60
                    );
                    println!(
                        "{} Time of expiration: {}",
                        "[#]".blue(),
                        current_time.format("%Y-%m-%d %H:%M:%S")
                    );
                    println!(
                        "{} Total monitoring duration: {} minutes",
                        "[#]".blue(),
                        self.total_wait_time.as_secs() / 60
                    );
                    break;
                }
                Err(e) => return Err(e),
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        Ok(())
    }
}