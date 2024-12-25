use crate::oauth;
use serde::Deserialize;

#[derive(Deserialize)]
struct CommodityItem {
    id: u64,
}
#[derive(Deserialize)]
struct CommodityAuction {
    id: u64,
    item: CommodityItem,
    quantity: u64,
    unit_price: u64,
    time_left: String,
}
#[derive(Deserialize)]
struct CommoditiesResponse {
    auctions: Vec<CommodityAuction>,
}

pub(crate) struct Scraper<'a> {
    http_client: reqwest::Client,
    token: &'a oauth::OAuthToken,
    base_url: String,
}

impl Scraper<'_> {
    pub(crate) fn new<'a>(token: &'a oauth::OAuthToken, base_url: &str) -> Scraper<'a> {
        Scraper {
            http_client: reqwest::Client::new(),
            token,
            base_url: base_url.to_string(),
        }
    }
    pub(crate) async fn fetch(&self, path: &str) -> Result<String, reqwest::Error> {
        let url = reqwest::Url::parse_with_params(
            format!("{}{}", self.base_url, path).as_str(),
            [("namespace", "dynamic-eu")],
        )
        .expect("URL parsing failed");
        let resp = self
            .http_client
            .get(url)
            .bearer_auth(&self.token.access_token)
            .send()
            .await;

        let text = match resp {
            Ok(resp) => match resp.text().await {
                Ok(text) => text,
                Err(e) => return Err(e),
            },
            Err(e) => return Err(e),
        };

        Ok(text)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    async fn demo_content() -> &'static str {
        let data = r#"
        {
          "_links": {
            "self": {
              "href": "https://eu.api.blizzard.com/data/wow/auctions/commodities?namespace=dynamic-eu"
            }
          },
          "auctions": [
            {
              "id": 2136385844,
              "item": {
                "id": 19304
              },
              "quantity": 3,
              "unit_price": 200,
              "time_left": "SHORT"
            },
            {
              "id": 2136385859,
              "item": {
                "id": 173160
              },
              "quantity": 1014,
              "unit_price": 1889700,
              "time_left": "SHORT"
            }
            ]
        }
        "#;
        data
    }

    #[tokio::test]
    async fn test_deserialization() {
        let contents = demo_content().await;
        let response: CommoditiesResponse = serde_json::from_str(contents).unwrap();
        assert_eq!(response.auctions.len(), 2);
        assert_eq!(response.auctions[0].id, 2136385844);
        assert_eq!(response.auctions[0].quantity, 3);
        assert_eq!(response.auctions[0].item.id, 19304);
    }
}
