use log::{debug, error, info};
use serde_json::Value;
use std::collections::HashMap;
use worker::{Error, Fetch, Method, Request, Response};

#[derive(Debug)]
pub struct TitoClient(pub Request);
impl TitoClient {
    const BASE_URI: &'static str = "https://api.tito.io/v3";
    pub fn new(resource: &str) -> Self {
        Self(
            Request::new(&format!("{}/{}", Self::BASE_URI, resource), Method::Get)
                .expect("Failed to get Request init'd"),
        )
    }

    pub fn set_token(&mut self, token: &str) {
        self.0
            .headers_mut()
            .expect("Unable to get headers_mut from Request object.")
            .set("Authorization", format!("Token token={}", token).as_str())
            .expect("Unable to set Authorization header on Request object.")
    }

    fn prepare(&mut self) {
        self.0
            .headers_mut()
            .expect("Unable to get headers_mut from Request object.")
            .set("Accept", "application/json")
            .expect("Unable to set Accept header on Request object.");

        self.0
            .headers_mut()
            .expect("Unable to get headers_mut from Request object.")
            .set("Content-Type", "application/json")
            .expect("Unable to set Content-Type header on Request object.");
    }

    pub async fn dispatch(&mut self) -> Result<Response, Error> {
        self.prepare();
        Fetch::Request(self.0.clone().unwrap()).send().await
    }
}

pub struct TestTitoToken;

impl TestTitoToken {
    pub async fn run(token: &str) -> bool {
        let mut client = TitoClient::new("/hello");
        client.set_token(token);
        if let Ok(resp) = client.dispatch().await {
            resp.status_code() == 200
        } else {
            false
        }
    }
}
pub struct GetUpcomingEventTicketCount;

impl GetUpcomingEventTicketCount {
    pub async fn run(token: &str, account_slug: &str) -> Result<i64, Error> {
        info!("Running GetUpcomingEventTicketCount method...");

        debug!("Running with token: {}", token);
        debug!("Running with account_slug: {}", account_slug);
        debug!("Creating client & setting resource");

        let mut client = TitoClient::new(&format!("{}/events?view=extended", account_slug));

        debug!("Setting token");

        client.set_token(token);

        let mut resp = match client.dispatch().await {
            Ok(resp) => resp,
            Err(err) => {
                error!("Error getting response: {:?}", err);
                return Err(err);
            }
        };
        let json = resp
            .text()
            .await
            .expect("Unable to get JSON from response object.");

        let map: HashMap<String, Value> =
            serde_json::from_str(&json).expect("Unable to parse JSON response string");
        debug!("Status code: {}", resp.status_code());
        let tickets_count = map
            .get("events")
            .expect("Unable to get `events` key from JSON response")
            .get(0)
            .expect("Unable to get first `events` item from JSON response")
            .get("releases")
            .expect("Unable to get `releases` key from JSON response")
            .get(0)
            .expect("Unable to get first `releases` item from JSON response")
            .get("tickets_count")
            .expect("Unable to get `tickets_count` key from JSON response")
            .as_i64()
            .expect("Unable to convert `tickets_count` to i64 from JSON response");

        info!("Returning tickets_count");
        Ok(tickets_count)
    }
}
