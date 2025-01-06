mod oauth;
mod scrape;
mod storage;
use argh::FromArgs;
use storage::Storage;

trait Command {
    async fn run(&self);
}
#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
struct Cli {
    #[argh(subcommand)]
    cmd: SubcommandEnum,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubcommandEnum {
    Scrape(ScrapeSubcommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Query the commodities API.
#[argh(subcommand, name = "scrape")]
struct ScrapeSubcommand {
    #[argh(option)]
    /// where to store the API response.
    target: String,

    #[argh(option, default = "String::from(\"/auctions/commodities\")")]
    /// API endpoint to scrape, defaults to "/auctions/commodities"
    endpoint: String,
}

impl Command for ScrapeSubcommand {
    async fn run(&self) {
        let storage = storage::new_storage(self.target.as_str());

        let mut client = oauth::OAuthClient::from_env();
        let token = client.get_token().await;
        let scraper = scrape::Scraper::new(token, "https://eu.api.blizzard.com/data/wow");
        let response = scraper
            .fetch("/auctions/commodities")
            .await
            .expect("Fetch failed");

        let payload = storage::PayloadInfo::from_raw_bytes(response.as_bytes());

        storage.store(&payload).await;
    }
}

#[tokio::main]
async fn main() {
    let args: Cli = argh::from_env();
    match args.cmd {
        SubcommandEnum::Scrape(cmd) => cmd.run().await,
    }
}
