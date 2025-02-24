use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::{prelude::FromRow, SqlitePool};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let conn_res = SqlitePool::connect("sqlite::memory:").await;

    let conn = match conn_res {
        Ok(c) => c,
        Err(e) => {
            println!("Error opening a new database: {}", e);
            return;
        }
    };

    let table_create = sqlx::query(
        "CREATE TABLE files (id TEXT, chunk_num INT, hash BLOB, data MEDIUMBLOB, file_name TEXT, total_chunks INT)",
    )
    .execute(&conn)
    .await;

    match table_create {
        Ok(_) => println!("Created tables, OK"),
        Err(e) => {
            println!("Error creating tables {}", e);
            return;
        }
    };

    let s_state = ServerState { conn };

    let app = Router::new()
        .route("/", get(root))
        .route("/create", post(handle_post))
        .route("/download/{id}", get(handle_file_req))
        .route("/download/{id}/{chunk_num}", get(handle_range_req))
        .with_state(s_state);
    let addr = "0.0.0.0:3030";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Server running on {}", addr);
    axum::serve::serve(listener, app).await.unwrap();
}
#[derive(Clone)]
struct ServerState {
    conn: SqlitePool,
}

async fn root() -> &'static str {
    "Hi from the partage server!"
}

async fn handle_post(State(state): State<ServerState>, mut multipart: axum::extract::Multipart) {
    let mut text_map: HashMap<&str, String> = HashMap::new();
    let mut byte_map: HashMap<&str, Vec<u8>> = HashMap::new();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        match name.as_str() {
            "file_name" => {
                text_map.insert(
                    "file_name",
                    String::from_utf8(data.to_vec()).unwrap().to_owned(),
                );
            }
            "id" => {
                text_map.insert("id", String::from_utf8(data.to_vec()).unwrap());
            }
            "chunk_num" => {
                text_map.insert("chunk_num", String::from_utf8(data.to_vec()).unwrap());
            }
            "hash" => {
                byte_map.insert("hash", data.to_vec());
            }
            "file" => {
                byte_map.insert("file", data.to_vec());
            }
            "total_chunks" => {
                text_map.insert("total_chunks", String::from_utf8(data.to_vec()).unwrap());
            }
            _ => {
                println!("Error parsing!");
            }
        }
    }

    let id = text_map.get("id").unwrap();
    let file_name = text_map.get("file_name").unwrap();

    let hash = byte_map.get("hash").unwrap();
    let data = byte_map.get("file").unwrap();
    let chunk_num = text_map.get("chunk_num").unwrap().parse::<i32>().unwrap();
    let total_chunks = text_map
        .get("total_chunks")
        .unwrap()
        .parse::<i32>()
        .unwrap();

    let insert = sqlx::query("INSERT INTO files VALUES (?, ?, ?, ?, ?, ?)")
        .bind(id)
        .bind(chunk_num)
        .bind(hash)
        .bind(data)
        .bind(file_name)
        .bind(total_chunks)
        .execute(&state.conn)
        .await;

    match insert {
        Ok(_) => (),
        Err(e) => println!("Error updating database: {}", e),
    }
}

#[derive(FromRow, Debug)]
struct FileData {
    id: String,
    chunk_num: i32,
    hash: Vec<u8>,
    data: Vec<u8>,
    file_name: String,
    total_chunks: i32,
}
async fn handle_file_req(
    State(state): State<ServerState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let rows = sqlx::query_as::<_, FileData>("SELECT * FROM FILES where id = ?")
        .bind(&id)
        .fetch_optional(&state.conn)
        .await
        .unwrap();
    match rows {
        Some(r) => Json(json!({"file_name": r.file_name, "total_chunks": r.total_chunks})),
        None => Json(json!("")),
    }
}

async fn handle_range_req(
    State(state): State<ServerState>,
    Path((id, chunk_num)): Path<(String, i32)>,
) -> Response<Body> {
    let rows = sqlx::query_as::<_, FileData>("SELECT * FROM files where id = ? AND chunk_num = ?")
        .bind(&id)
        .bind(chunk_num)
        .fetch_optional(&state.conn)
        .await
        .unwrap();

    match rows {
        Some(r) => Response::builder().body(Body::from(r.data)).unwrap(),
        None => Response::builder().body(Body::from(())).unwrap(),
    }
}
