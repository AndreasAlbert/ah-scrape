use std::path;
use tokio::time::Instant;

mod oauth;
mod scrape;
mod storage;
use storage::Storage;
#[tokio::main]
async fn main() {
    let mut client = oauth::OAuthClient::from_env();
    let token = client.get_token().await;
    println!("Token: {:?} ", token);

    let scraper = scrape::Scraper::new(token, "https://eu.api.blizzard.com/data/wow");
    let response = scraper
        .fetch("/auctions/commodities")
        .await
        .expect("Fetch failed");

    let storage = storage::LocalStorage::new(path::Path::new("/tmp/dump"));
    storage.store(response.as_str()).await;
}
