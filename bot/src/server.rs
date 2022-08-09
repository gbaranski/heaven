use std::{net::SocketAddr, sync::Arc};

use crate::{
    database::Database,
    models::{Angel, AngelID},
    store::{Authorization, Store},
};
use axum::{extract::{Path, Query}, http::StatusCode, routing::get, Extension, Json, Router};
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

pub async fn run(address: SocketAddr, database: Database, store: Store, discord: Http) {
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
        .route("/authorize/:angel_id", get(authorize_angel))
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
    let angel = app_state
        .database
        .get_angel_by_minecraft_name(&minecraft_name);
    Json(angel)
}

async fn authorize_angel(
    Extension(app_state): Extension<AppState>,
    Path(angel_id): Path<AngelID>,
    Query(from): Query<SocketAddr>,
) -> StatusCode {
    let angel = app_state.database.get_angel_by_id(&angel_id);
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
    dm_channel
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
            }).content(format!("New login request for Minecraft server from {from}. If that's not you, please click Deny"))
        })
        .await
        .unwrap();
    let authorization = authorization.await;
    match authorization {
        Authorization::Allow => StatusCode::OK,
        Authorization::Deny => StatusCode::UNAUTHORIZED,
    }
}
