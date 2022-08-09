use std::net::SocketAddr;

use crate::{database::Database, models::Angel};
use axum::{routing::get, Extension, Json, Router, extract::Path};

#[derive(Debug, Clone)]
struct AppState {
    database: Database,
}

pub async fn run(address: SocketAddr, database: Database) {
    let app_state = AppState { database };
    let app = Router::new()
        .route("/angel/by-minecraft-name/:minecraft_name", get(get_angel))
        .layer(Extension(app_state));
    tracing::debug!("listening on {}", address);
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_angel(
    Extension(app_state): Extension<AppState>,
    Path(minecraft_name): Path<String>,
) -> Json<Option<Angel>> {
    let user = app_state
        .database
        .get_angel_by_minecraft_name(&minecraft_name);
    Json(user)
}
