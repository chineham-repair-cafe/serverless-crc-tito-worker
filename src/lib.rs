mod ext;

use ext::tito::client::{GetUpcomingEventTicketCount, TestTitoToken};
use log::{error, info, debug};
use serde_json::json;
use worker::*;

#[derive(Clone, Debug)]
pub struct State {
    pub token: String,
    pub account_slug: String,
}

fn cors_response_header(req: &worker::Headers, origin: &str) -> worker::Headers {
    let mut headers = worker::Headers::new();
    let origin = match req.get("Origin").expect("Failed to get origin") {
        Some(value) => value,
        None => return headers,
    };

    headers
        .set("Access-Control-Allow-Headers", "Content-Type")
        .expect("Unable to set header");
    headers
        .set("Access-Control-Allow-Methods", "GET")
        .expect("Unable to set header");
    headers.set("Vary", "Origin").expect("Unable to set header");

    if origin.split(',').any(|val| val == origin) {
        headers
            .set("Access-Control-Allow-Origin", &origin)
            .expect("Unable to set header");
    }

    headers
        .set("Access-Control-Max-Age", "86400")
        .expect("Unable to set header");
    headers
}

async fn get_ticket_count(state: State, req: Request) -> Result<Response> {
    let token = state.token;
    let account_slug = state.account_slug;

    debug!("Fetching ticket count");
    let tickets_count = match GetUpcomingEventTicketCount::run(&token, &account_slug).await {
        Ok(count) => {
            debug!("Ticket count: {}", count);
            count
        }
        Err(_) => {
            error!("Failed to fetch ticket count");
            return Response::error(json!({"status": "BAD_COUNT"}).to_string(), 500);
        }
    };

    let json = json!({
        "count": tickets_count,
    });

    let headers = cors_response_header(&req.headers(), "https://nhrc.uk");

    Ok(Response::ok(json.to_string())?.with_headers(headers))
}

#[event(start)]
pub fn start() {
    worker_logger::init_with_level(&log::Level::Info)
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    info!("Starting up");

    // Env exist checks.
    if env.secret("TITO_TOKEN").is_err() {
        panic!("TITO_TOKEN not found in environment");
    }

    if env.secret("TITO_ACCOUNT_SLUG").is_err() {
        panic!("TITO_ACCOUNT_SLUG not found in environment");
    }

    if env.var("TITO_TOKEN_CHECK").is_err() {
        panic!("TITO_TOKEN_CHECK not found in environment");
    }

    let state: State = State {
        token: env.secret("TITO_TOKEN").unwrap().to_string(),
        account_slug: env.secret("TITO_ACCOUNT_SLUG").unwrap().to_string(),
    };

    if env.var("TITO_TOKEN_CHECK").unwrap().to_string() == "true" {
        debug!("Running token test");
        if !TestTitoToken::run(&state.token).await {
            error!("Token test failed");
            let json = json!({
                "status": "BAD_CONF",
            });
            return Response::error(json.to_string(), 500);
        };
    };

    let strip_pattern = &format!("https://{}", &req.headers().get("host")
            .expect("Failed to get host")
            .expect("Failed to get host"));
    let stripped_url = req
        .url()
        .expect("Failed to get request url")
        .clone()
        .to_string()
        .replace(strip_pattern, "")
        .to_lowercase();

    return match stripped_url.clone().as_str() {
        "/tickets/count" => return get_ticket_count(state.clone()).await,
        "/events/next" | _ => {
            let json = json!({
                "status": "NOT_FOUND",
            });
            Response::error(json.to_string(), 404)
        }
    };
}
