//! This project is split in 2 main modules:
//!
//! - [gateway] (external gateway implementation)
//! - [connect] (gateway.connect API surface)
#![doc = include_str!("../README.md")]

use std::net::{Ipv4Addr, SocketAddrV4};

use axum::Router;
use tracing_subscriber::EnvFilter;

/// Implementation of `gateway.connect`
///
/// This module defines the types and endpoints to communicate with the `Gateway.Connect` API.
mod connect;

mod db;
/// Gateway integration implementation
///
/// This module defines the types and methods to communicate with an external gateway. In this case it is SeguraPay
mod gateway;
mod state;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(false)
        .init();

    match dotenvy::dotenv() {
        Ok(p) => tracing::info!(path = %p.display(), "Loaded environment variables from .env file"),
        Err(e) => tracing::warn!("Failed to environment variables from .env: {e}"),
    };
    let db = db::Db::connect().await.expect("database is not available");
    let state = state::AppState::new(db);

    let app = Router::new()
        .merge(connect::api::router())
        .nest("/gateway", gateway::api::router())
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3030);

    let listener = tokio::net::TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port))
        .await
        .unwrap();

    tracing::info!("Serving on port {port}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .unwrap();
}
