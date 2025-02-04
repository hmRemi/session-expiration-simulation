use colored::*;
use dotenv::dotenv;
use session_monitor::{ChromeDriver, Config, SessionMonitor};

#[tokio::main]
async fn main() -> Result<(), session_monitor::SessionError> {
    dotenv().ok();

    std::env::set_var("TF_CPP_MIN_LOG_LEVEL", "3");

    let config = Config::from_env()?;

    println!(
        "{} Checking if ChromeDriver is running at {}",
        "[#]".blue(),
        config.webdriver_url
    );
    ChromeDriver::check_ready(&config.webdriver_url).await?;

    println!("{} Initializing session monitor...", "[?]".yellow());
    let mut monitor = SessionMonitor::new(config).await?;

    println!("{} Starting login sequence...", "[?]".yellow());
    monitor.login().await?;

    println!("{} Beginning monitoring cycle...", "[?]".yellow());
    monitor.run().await?;

    Ok(())
}
