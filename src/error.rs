use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Environment variable not found: {0}")]
    EnvVarMissing(String),
    #[error("WebDriver error: {0}")]
    WebDriver(#[from] fantoccini::error::NewSessionError),
    #[error("Session expired")]
    SessionExpired,
    #[error("Browser interaction failed: {0}")]
    BrowserError(#[from] fantoccini::error::CmdError),
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("ChromeDriver process error: {0}")]
    ProcessError(#[from] io::Error),
}