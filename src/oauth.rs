use base64::prelude::*;
use serde::Deserialize;
use std::time::{Duration, Instant};
#[derive(Debug)]
pub(crate) struct OAuthToken {
    pub(crate) access_token: String,
    pub(crate) expiration: Instant,
}
impl OAuthToken {
    fn is_valid(&self) -> bool {
        self.expiration >= Instant::now()
    }
}
#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

fn token_from_response(token: TokenResponse) -> OAuthToken {
    OAuthToken {
        access_token: token.access_token,
        expiration: Instant::now() + Duration::from_secs(token.expires_in - 60),
    }
}

pub(crate) struct OAuthClient {
    http_client: reqwest::Client,
    client_id: String,
    client_secret: String,
    refresh_url: String,
    _token: Option<OAuthToken>,
}

impl OAuthClient {
    fn new(
        http_client: reqwest::Client,
        client_id: &str,
        client_secret: &str,
        refresh_url: &str,
    ) -> Self {
        OAuthClient {
            http_client,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            refresh_url: refresh_url.to_string(),
            _token: None,
        }
    }

    pub(crate) fn from_env() -> Self {
        let client_id = std::env::var("CLIENT_ID").unwrap();
        let client_secret = std::env::var("CLIENT_SECRET").unwrap();
        let refresh_url = std::env::var("REFRESH_URL").unwrap();
        OAuthClient::new(
            reqwest::Client::new(),
            client_id.as_str(),
            client_secret.as_str(),
            refresh_url.as_str(),
        )
    }

    pub(crate) async fn get_token(&mut self) -> &OAuthToken {
        if self._token.is_none() {
            self.refresh_token().await;
        }
        self._token.as_ref().unwrap()
    }

    async fn refresh_token(&mut self) {
        let res = self
            .http_client
            .post(self.refresh_url.as_str())
            .basic_auth(self.client_id.as_str(), Some(self.client_secret.as_str()))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await;

        match res {
            Ok(res) => {
                let text = res.text().await;
                match text {
                    Ok(text) => {
                        let token_response: TokenResponse =
                            serde_json::from_str(text.as_str()).unwrap();
                        self._token = Some(token_from_response(token_response));
                    }
                    Err(e) => {
                        panic!("Failed to parse token response: {:?}", e);
                    }
                }
            }
            Err(e) => {
                panic!("Failed to refresh token: {:?}", e);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_token_from_response() {
        let token_reponse = TokenResponse {
            access_token: "token".to_string(),
            expires_in: 3600,
        };
        let token = token_from_response(token_reponse);
        assert_eq!(token.access_token, "token");
        assert!(token.is_valid());
    }

    #[tokio::test]
    async fn test_token_expired() {
        let token = OAuthToken {
            access_token: "token".to_string(),
            expiration: Instant::now() - Duration::from_secs(60),
        };
        assert!(!token.is_valid());
    }

    #[tokio::test]
    async fn test_get_token() {
        // Arrange
        let mut server = mockito::Server::new_async().await;
        let url = server.url();
        let route = "/refresh";
        let client_id = "client_id";
        let client_secret = "client_secret";
        let mock = server
            .mock("POST", route)
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"access_token": "token", "expires_in": 3600, "extra_key": "extra_value"}"#,
            )
            .match_header(
                "authorization",
                format!(
                    "Basic {}",
                    BASE64_STANDARD.encode(format!("{}:{}", client_id, client_secret).as_bytes())
                )
                .as_str(),
            )
            .expect(1)
            .create_async()
            .await;
        let full_url = format!("{}{}", url, route);

        let mut client = OAuthClient::new(
            reqwest::Client::new(),
            client_id,
            client_secret,
            full_url.as_str(),
        );

        // Act
        let token = client.get_token().await;

        // Assert
        mock.assert();
        assert!(token.is_valid());
        assert_eq!(token.access_token, "token");
    }
}
