use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let files = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        .route("/", get(root))
        .route("/create", post(handle_post))
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

struct MemFile {
    file: Vec<u8>,
    file_name: String,
    hash: Vec<u8>,
}

async fn handle_post(
    State(state): State<Arc<Mutex<HashMap<String, MemFile>>>>,

    Json(payload): Json<RequestData>,
) -> axum::http::StatusCode {
    println!("received post with id: {}\n", payload.id);

    println!("length of file: {}", payload.file.len());

    println!("length of hash: {}", payload.hash.len());

    let mut map = state.lock().await;
    map.insert(
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
    State(state): State<Arc<Mutex<HashMap<String, MemFile>>>>,
    axum::extract::Path(file_id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let map = state.lock().await;

    if !map.contains_key(&file_id) {
        return Json(json!(""));
    }

    let file_info = map.get(&file_id).unwrap();

    let json =
        json!({"file_name": file_info.file_name, "file": file_info.file, "hash": file_info.hash});

    Json(json)
}
