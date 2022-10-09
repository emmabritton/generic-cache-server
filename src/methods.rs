use std::collections::HashMap;
use std::ops::Add;
use std::sync::{Arc};
use axum::{Extension, Json};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::{Duration, Utc};
use log::{debug, trace};
use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::json;
use tokio::sync::RwLock;
use crate::Cache;
use crate::cache::CacheEntry;
use crate::utils::AppError;

type ExtCache = Extension<Arc<RwLock<Cache>>>;
type ExtTokens = Extension<Arc<Vec<String>>>;

pub async fn alive() -> impl IntoResponse {
    env!("CARGO_PKG_VERSION").to_string()
}

pub async fn stats(Extension(cache): ExtCache) -> Result<impl IntoResponse, AppError> {
    let json = cache.read().await.all()?;
    Ok((StatusCode::OK, Json(json)))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Clear {
    pub token: String,
    pub key: String,
}

pub async fn clear(Json(payload): Json<Clear>, Extension(cache): ExtCache, Extension(tokens): ExtTokens) -> Result<impl IntoResponse, AppError> {
    if tokens.contains(&payload.token) {
        cache.write().await.clear(&payload.key);
        Ok((StatusCode::OK, "Done"))
    } else {
        Ok((StatusCode::UNAUTHORIZED, "Bad token"))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub token: String,
    pub url: String,
    pub key: String,
    pub headers: HashMap<String, String>,
    pub method: String,
    pub body: Option<String>,
    pub ttl: u64,
}

pub async fn send_request(Json(payload): Json<Request>, Extension(cache): ExtCache, Extension(tokens): ExtTokens) -> Result<impl IntoResponse, AppError> {
    if tokens.contains(&payload.token) {
        let entry = cache.write().await.get(&payload.key).cloned();
        match entry {
            Some(entry) => {
                debug!("Returning from cache for {}", payload.key);
                Ok((StatusCode::OK, Json(entry.as_json()?)))
            }
            None => {
                debug!("Sending {} request to {}", payload.method, payload.url);
                let client = Client::new();
                let mut request = match payload.method.as_str() {
                    "get" => client.get(payload.url),
                    "post" => client.post(payload.url),
                    _ => return Ok((StatusCode::BAD_REQUEST, Json(json!({"message":"Unknown/invalid method"}))))
                };

                if let Some(body) = payload.body {
                    request = request.json(&body);
                }
                for (name, value) in payload.headers {
                    request = request.header(name, value)
                }
                trace!("Sending request..");
                let response = request.send().await?;
                trace!("Reading response..");
                let response_body = response.text().await?;
                let created_at = Utc::now().naive_utc().add(Duration::seconds(payload.ttl as i64));
                let entry = CacheEntry::new(payload.key, response_body.clone(), created_at, payload.token);
                trace!("Storing result");
                cache.write().await.insert(entry);
                Ok((StatusCode::OK, Json(serde_json::from_str(&response_body)?)))
            }
        }
    } else {
        Ok((StatusCode::UNAUTHORIZED, Json(json!({"message":"Bad token"}))))
    }
}