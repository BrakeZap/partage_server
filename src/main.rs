use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use moka::sync::Cache;
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, panic::UnwindSafe, process::Termination, time::Duration};

#[tokio::main]
async fn main() {
    //let files: moka::sync::Cache<String, MemFile> = Cache::builder()
    //    .time_to_live(Duration::from_secs(60 * 60))
    //    .build();

    let s_state = ServerState {
        uploaded_files: Cache::builder()
            .time_to_live(Duration::from_secs(60 * 60))
            .build(),
        temp_files: Cache::builder().build(),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/create", post(handle_post))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 200))
        .route("/download/{id}", get(handle_download))
        .with_state(s_state);
    let addr = "0.0.0.0:3030";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Server running on {}", addr);
    axum::serve::serve(listener, app).await.unwrap();
}
#[derive(Clone)]
struct ServerState {
    uploaded_files: Cache<String, MemFile>,
    temp_files: Cache<String, Vec<Vec<u8>>>,
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

//async fn handle_post(
//    State(state): State<Cache<String, MemFile>>,
//
//    Json(payload): Json<RequestData>,
//) -> axum::http::StatusCode {
//    println!("received post with id: {}\n", payload.id);
//
//    println!("length of file: {}", payload.file.len());
//
//    println!("length of hash: {}", payload.hash.len());
//
//    state.insert(
//        payload.id,
//        MemFile {
//            file_name: payload.file_name,
//            file: payload.file,
//            hash: payload.hash,
//        },
//    );
//    StatusCode::OK
//}

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

    println!("text map {:?}", text_map);

    //println!("byte map {:?}", byte_map);

    let id = text_map.get("id").unwrap();

    if state.temp_files.contains_key(id) {
        let mut new_vec = state.temp_files.get(id).unwrap();
        new_vec.push(byte_map.get("file").unwrap().to_vec());
        state.temp_files.insert(id.to_string(), new_vec);
    } else {
        let mut temp_vec: Vec<Vec<u8>> = Vec::new();
        temp_vec.push(byte_map.get("file").unwrap().to_vec());
        state.temp_files.insert(id.to_string(), temp_vec);
    }

    if text_map.get("chunk_num").unwrap().parse::<usize>().unwrap()
        == text_map
            .get("total_chunks")
            .unwrap()
            .parse::<usize>()
            .unwrap()
            - 1
    {
        let mut final_file: Vec<u8> = Vec::new();
        let temp = state.temp_files.get(id).unwrap();
        println!("Length of temp files: {}", temp.len());
        for i in 0..temp.len() {
            let curr = temp.get(i).unwrap();
            for j in 0..curr.len() {
                final_file.push(*curr.get(j).unwrap());
            }
        }

        println!("Id: {} has total length of: {}", id, final_file.len());
        state.uploaded_files.insert(
            id.to_string(),
            MemFile {
                file_name: text_map.get("file_name").unwrap().to_string(),
                hash: byte_map.get("hash").unwrap().to_vec(),
                file: final_file,
            },
        );
    }
}

async fn handle_download(
    State(state): State<ServerState>,
    axum::extract::Path(file_id): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    if !state.uploaded_files.contains_key(&file_id) {
        return Json(json!(""));
    }

    let file_info = state.uploaded_files.get(&file_id).unwrap();

    let json =
        json!({"file_name": file_info.file_name, "file": file_info.file, "hash": file_info.hash});

    Json(json)
}
//
//async fn handle_download(
//    State(state): State<ServerState>,
//    axum::extract::Path(file_id): axum::extract::Path<String>,
//) {
//    todo!()
//}
