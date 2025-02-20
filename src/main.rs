use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/create", post(handle_post));
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
    hash: Vec<u8>,
}

#[derive(Serialize)]
struct ResponseData {
    response: String,
}

async fn handle_post(Json(payload): Json<RequestData>) -> Json<ResponseData> {
    Json(ResponseData {
        response: format!("Created file."),
    })
}
