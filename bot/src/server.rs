use std::{net::SocketAddr, sync::Arc};

use crate::{
    database::Database,
    models::Angel,
    store::{Authorization, Store},
};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use axum_server::Handle;
use serde::Deserialize;
use serenity::{http::Http, model::prelude::component::ButtonStyle};
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

#[derive(Debug, Clone)]
struct AppState {
    database: Database,
    store: Store,
    discord: Arc<Http>,
}

pub fn run(address: SocketAddr, database: Database, store: Store, discord: Http) -> Handle {
    let app_state = AppState {
        database,
        store,
        discord: Arc::new(discord),
    };
    let app = Router::new()
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
    let handle = Handle::new();
    tokio::spawn({
        let handle = handle.clone();
        async move {
            tracing::debug!("listening on {}", address);
            axum_server::bind(address)
                .handle(handle)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
    });
    handle
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
    let user = app_state
        .discord
        .get_user(angel.discord_id.0)
        .await
        .unwrap();
    let dm_channel = user.create_dm_channel(&app_state.discord).await.unwrap();
    let authorization = app_state.store.get_authorization(angel.discord_id);
    let mut message = dm_channel
        .send_message(&app_state.discord, |f| {
            f.components(|c| {
                c.create_action_row(|ar| {
                    ar.create_button(|b| {
                        b.custom_id("authorization/allow")
                            .emoji('✅')
                            .label("Allow")
                            .style(ButtonStyle::Primary)
                    })
                    .create_button(|b| {
                        b.custom_id("authorization/deny")
                            .emoji('❌')
                            .label("Deny")
                            .style(ButtonStyle::Secondary)
                    })
                })
            })
            .content(format!(
                "New login request for Minecraft server from {from}."
            ))
        })
        .await
        .unwrap();
    let authorization = authorization.await;
    message
        .edit(&app_state.discord, |f| {
            f.components(|f| f.set_action_rows(vec![]))
        })
        .await
        .unwrap();
    match authorization {
        Authorization::Allow => StatusCode::OK,
        Authorization::Deny => StatusCode::UNAUTHORIZED,
    }
}
