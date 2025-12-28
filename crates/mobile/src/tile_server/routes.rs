use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use pmtiles::Compression;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use super::PmTilesReader;

type SharedReader = Arc<RwLock<PmTilesReader>>;

#[derive(Clone)]
pub struct AppState {
    reader: SharedReader,
    port: u16,
}

pub fn create_router(reader: SharedReader, port: u16) -> Router {
    let state = AppState { reader, port };
    Router::new()
        .route("/tiles/{z}/{x}/{y}/tile.pbf", get(serve_tile))
        .route("/tiles.json", get(serve_tilejson))
        .route("/health", get(health))
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(state)
}

async fn serve_tile(
    State(state): State<AppState>,
    Path((z, x, y)): Path<(u8, u64, u64)>,
) -> Response {
    let reader = state.reader.read().await;

    match reader.get_tile(z, x, y).await {
        Ok(Some(tile)) => {
            let mut response = (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/x-protobuf")],
                tile.to_vec(),
            )
                .into_response();

            if let Some(encoding) = compression_to_encoding(reader.get_header().tile_compression) {
                response
                    .headers_mut()
                    .insert(header::CONTENT_ENCODING, encoding);
            }

            response
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn compression_to_encoding(compression: Compression) -> Option<HeaderValue> {
    match compression {
        Compression::Gzip => Some(HeaderValue::from_static("gzip")),
        Compression::Brotli => Some(HeaderValue::from_static("br")),
        Compression::Zstd => Some(HeaderValue::from_static("zstd")),
        _ => None,
    }
}

async fn serve_tilejson(State(state): State<AppState>) -> Json<serde_json::Value> {
    let reader = state.reader.read().await;
    let header = reader.get_header();

    Json(serde_json::json!({
        "tilejson": "3.0.0",
        "tiles": [format!("http://localhost:{}/tiles/{{z}}/{{x}}/{{y}}/tile.pbf", state.port)],
        "minzoom": header.min_zoom,
        "maxzoom": header.max_zoom
    }))
}

async fn health() -> &'static str {
    "OK"
}
