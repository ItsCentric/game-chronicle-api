use std::{env, error};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use warp::{reject, reply, Rejection, Reply};

#[derive(Debug)]
struct InternalServerError;

impl reject::Reject for InternalServerError {}

#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expires_in: i32,
    pub token_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DumpResponse {
    #[serde(rename(deserialize = "s3_url"))]
    pub url: String,
}

async fn authenticate_with_twitch() -> Result<String, Box<dyn error::Error>> {
    let client = Client::new();
    let response = client
        .post(&format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials", env::var("TWITCH_CLIENT_ID")?, env::var("TWITCH_CLIENT_SECRET")?))
        .header("Content-Type", "application/json")
        .send()
        .await?;
    let json_response: AccessTokenResponse = response.json().await?;
    Ok(json_response.access_token)
}

pub async fn get_endpoint_csv_dump(endpoint: String) -> Result<impl Reply, Rejection> {
    let client = Client::new();
    let access_token = match authenticate_with_twitch().await {
        Ok(access_token) => access_token,
        Err(e) => {
            eprintln!("Error authenticating with Twitch: {}", e);
            return Err(reject::custom(InternalServerError));
        }
    };
    let client_id = match env::var("TWITCH_CLIENT_ID") {
        Ok(client_id) => client_id,
        Err(e) => {
            eprintln!("Error getting Twitch Client ID: {}", e);
            return Err(reject::custom(InternalServerError));
        }
    };
    let response = match client
        .get(&format!("https://api.igdb.com/v4/dumps/{}", endpoint))
        .header("Client-ID", client_id)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            println!("{}", e);
            return Err(reject::custom(InternalServerError));
        }
    };
    if response.status() == 404 {
        return Err(reject::not_found());
    }
    let json_response = match response.json::<DumpResponse>().await {
        Ok(json_response) => json_response,
        Err(e) => {
            eprintln!("Error deserializing dump response: {}", e);
            return Err(reject::custom(InternalServerError));
        }
    };
    Ok(reply::json(&json_response))
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if err.is_not_found() {
        Ok(reply::with_status(
            "NOT_FOUND",
            warp::http::StatusCode::NOT_FOUND,
        ))
    } else {
        Ok(reply::with_status(
            "INTERNAL_SERVER_ERROR",
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
