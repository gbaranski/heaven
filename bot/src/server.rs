use std::net::SocketAddr;

use crate::{database::Database, models::Angel};
use axum::{routing::get, Extension, Json, Router, extract::Path};
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

#[derive(Debug, Clone)]
struct AppState {
    database: Database,
}

pub async fn run(address: SocketAddr, database: Database) {
    let app_state = AppState { database };
    let app = Router::new()
        .route("/angel/by-minecraft-name/:minecraft_name", get(get_angel_by_minecraft_name))
        .layer(Extension(app_state))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        );
    tracing::debug!("listening on {}", address);
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_angel_by_minecraft_name(
    Extension(app_state): Extension<AppState>,
    Path(minecraft_name): Path<String>,
) -> Json<Option<Angel>> {
    let user = app_state
        .database
        .get_angel_by_minecraft_name(&minecraft_name);
    Json(user)
}
