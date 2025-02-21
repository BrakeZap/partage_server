use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use moka::sync::Cache;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let files = Cache::builder()
        .time_to_live(Duration::from_secs(60 * 60))
        .build();

    let app = Router::new()
        .route("/", get(root))
        .route("/create", post(handle_post))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 200))
        .route("/download/{id}", get(handle_download))
        .with_state(files);
    let addr = "0.0.0.0:3030";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Server running on {}", addr);
    axum::serve::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hi from the partage server!"
}
#[derive(Deserialize)]
struct RequestData {
    id: String,
    file: Vec<u8>,
    file_name: String,
    hash: Vec<u8>,
}
#[derive(Clone)]
struct MemFile {
    file: Vec<u8>,
    file_name: String,
    hash: Vec<u8>,
}

async fn handle_post(
    State(state): State<Cache<String, MemFile>>,

    Json(payload): Json<RequestData>,
) -> axum::http::StatusCode {
    println!("received post with id: {}\n", payload.id);

    println!("length of file: {}", payload.file.len());

    println!("length of hash: {}", payload.hash.len());

    state.insert(
        payload.id,
        MemFile {
            file_name: payload.file_name,
            file: payload.file,
            hash: payload.hash,
        },
    );
    StatusCode::OK
}

async fn handle_download(
    State(state): State<Cache<String, MemFile>>,
    axum::extract::Path(file_id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    if !state.contains_key(&file_id) {
        return Json(json!(""));
    }

    let file_info = state.get(&file_id).unwrap();

    let json =
        json!({"file_name": file_info.file_name, "file": file_info.file, "hash": file_info.hash});

    Json(json)
}
