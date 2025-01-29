use dotenv::dotenv;
use fantoccini::{Client, ClientBuilder, Locator};
use reqwest::Url;
use std::env;
use std::time::Duration;
use fantoccini::wd::Capabilities;
use tokio;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let login_url = env::var("LOGIN_URL").unwrap_or_else(|_| panic!("LOGIN_URL not set"));
    let username = env::var("USER").unwrap_or_else(|_| panic!("USER not set"));
    let password = env::var("PASSWORD").unwrap_or_else(|_| panic!("PASSWORD not set"));

    let mut wait_time = Duration::from_secs(5 * 60);

    loop {
        let mut client = start_browser().await.unwrap();
        login(&mut client, &login_url, &username, &password)
            .await
            .unwrap();

        println!("waiting for {:?}...", wait_time);
        tokio::time::sleep(wait_time).await;

        if check_session(&mut client, &login_url).await {
            println!("session still valid after {:?}", wait_time);
            client.close().await.unwrap();
            wait_time += Duration::from_secs(2 * 60);
        } else {
            println!("session expired after {:?} lol", wait_time);
            break;
        }
    }
}

async fn start_browser() -> Result<Client, Box<dyn std::error::Error>> {
    let cap: Capabilities = serde_json::from_str(
        r#"{"browserName":"chrome","goog:chromeOptions":{"args":["--headless"]}}"#,
    )
    .unwrap();
    let client = ClientBuilder::native()
        //.capabilities(cap)
        .connect("http://localhost:9515")
        .await
        .expect("failed to connect to webdriver ffs");
    Ok(client)
}

async fn login(
    client: &mut Client,
    url: &str,
    username: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    client.goto(url).await?;

    let username_field = client
        .find(Locator::Css(
            "[name='ctl00$ContentPlaceHolder1$Brukernavn']",
        ))
        .await?;
    let password_field = client
        .find(Locator::Css("[name='ctl00$ContentPlaceHolder1$Passord']"))
        .await?;

    username_field.send_keys(username).await?;
    password_field.send_keys(password).await?;

    let submit_button = client.find(Locator::Css("input[type='submit']")).await?;
    submit_button.click().await?;

    client.wait().for_element(Locator::Css("body")).await?;
    client.goto(&format!("{}KommendeVakter.aspx", url)).await?;

    Ok(())
}

async fn check_session(client: &mut Client, login_url: &String) -> bool {
    if let Err(_) = refresh_site(client, login_url).await {
        return false;
    }
    true
}

async fn refresh_site(
    client: &mut Client,
    login_url: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    client.refresh().await?;

    let current_url = client.current_url().await?;
    let login_url_parsed = Url::parse(login_url)?;

    if current_url == login_url_parsed {
        return Err("change in url aka we got redirected back to login page".into());
    }

    Ok(())
}
