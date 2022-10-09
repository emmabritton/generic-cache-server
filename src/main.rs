use std::net::SocketAddr;
use std::sync::{Arc};
use axum::{Extension, Router, Server};
use axum::routing::{get, post};
use log::debug;
use crate::cache::Cache;
use crate::init::setup;
use anyhow::Result;
use tokio::sync::RwLock;
use crate::methods::{alive, clear, send_request, stats};

mod init;
mod cache;
mod utils;
mod methods;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting cache server...");

    let config = setup()?;

    debug!("Config read");

    let cache = Arc::new(RwLock::new(Cache::new()));

    let app = Router::new()
        .route("/alive", get(alive))
        .route("/request", post(send_request))
        .route("/clear", post(clear))
        .route("/stats", get(stats))
        .layer(Extension(Arc::new(config.keys)))
        .layer(Extension(cache));

    let addr = SocketAddr::from((config.ip_addr, config.port));

    debug!("Server starting on {addr}");

    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}
