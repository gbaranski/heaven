use std::{net::SocketAddr, sync::Arc};

use crate::{
    authorizations::Authorization, bot::DiscordBot, configuration::Configuration,
    database::Database, models::Angel,
};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use axum_server::Handle;
use miette::{IntoDiagnostic, Result};
use serde::Deserialize;
use tokio_graceful_shutdown::SubsystemHandle;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

#[derive(Debug, Clone)]
struct AppState {
    database: Database,
    discord_bot: DiscordBot,
}

pub struct Server {
    router: Router,
    configuration: Arc<Configuration>,
}

impl Server {
    pub fn new(
        database: Database,
        configuration: Arc<Configuration>,
        discord_bot: DiscordBot,
    ) -> Self {
        let app_state = AppState {
            database,
            discord_bot,
        };
        let router = Router::new()
            .route(
                "/angel/by-minecraft-name/:minecraft_name",
                get(get_angel_by_minecraft_name),
            )
            .route(
                "/angel/by-minecraft-name/:minecraft_name/authorize",
                post(authorize_angel_by_minecraft_name),
            )
            .route("/health-check", get(health_check))
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
        Self {
            router,
            configuration,
        }
    }

    pub async fn run(self, subsystem: SubsystemHandle) -> Result<()> {
        let handle = Handle::new();
        let address = SocketAddr::from(([0, 0, 0, 0], self.configuration.port));
        let future = axum_server::bind(address)
            .handle(handle)
            .serve(self.router.into_make_service());
        tokio::select! {
            result = future => {
                result.into_diagnostic()
            }
            _ = subsystem.on_shutdown_requested() => {
                Ok(())
            }
        }
    }
}

async fn health_check() -> &'static str {
    "Hello, World!"
}

async fn get_angel_by_minecraft_name(
    Extension(app_state): Extension<AppState>,
    Path(minecraft_name): Path<String>,
) -> Json<Option<Angel>> {
    let angel = app_state
        .database
        .get_angel_by_minecraft_name(&minecraft_name);
    Json(angel)
}

#[derive(Deserialize)]
struct AuthorizeQuery {
    from: std::net::IpAddr,
}

async fn authorize_angel_by_minecraft_name(
    Extension(app_state): Extension<AppState>,
    Path(minecraft_name): Path<String>,
    Query(AuthorizeQuery { from }): Query<AuthorizeQuery>,
) -> StatusCode {
    let angel = app_state
        .database
        .get_angel_by_minecraft_name(&minecraft_name);
    let angel = match angel {
        Some(angel) => angel,
        None => return StatusCode::UNAUTHORIZED,
    };
    let authorization = app_state
        .discord_bot
        .authorize(angel.discord_id, from)
        .await
        .unwrap();
    match authorization {
        Authorization::Allow => StatusCode::OK,
        Authorization::Deny => StatusCode::UNAUTHORIZED,
    }
}
