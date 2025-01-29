use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
};
use axum::{
    extract::{Path as AxumPath, State},
    response::IntoResponse,
    Json,
};
use minio_rsc::{client::ListObjectsArgs, Minio};
use regex::Regex;
use reqwest::Method;
use serde::Serialize;
use utils::get_file;
mod utils;

const FIRMWARE_DIR: &str = "bin";

#[derive(Serialize)]
struct Manifest {
    version: String,
    error: String,
}

// basic handler that responds with a static string
#[allow(dead_code)]
pub async fn root() -> &'static str {
    "Welcome to Charizhard OTA ! Check /latest to get latest firmware"
}

pub async fn handle_manifest(State(instance): State<Minio>) -> impl IntoResponse {
    let args = ListObjectsArgs::default();
    let query = instance.list_objects("bin", args).await;
    let re = Regex::new(r"charizhard\.V(\d+\.\d+)\.bin").unwrap();

    match query {
        Ok(res) => {
            let mut version_files: Vec<String> = res
                .contents
                .iter()
                .filter_map(|object| {
                    re.captures(&object.key).and_then(|caps| {
                        caps.get(1).map(|version| version.as_str().to_string())
                    })
                })
                .collect();

            version_files.sort();
            let latest_version = match version_files.last() {
                Some(vers) => vers,
                None => {
                    return (
                        StatusCode::NO_CONTENT,
                        Json(Manifest {
                            version: "".to_string(),
                            error: "No firmware files found".to_string(),
                        }),
                    )
                }
            };
            (
                StatusCode::OK,
                Json(Manifest {
                    version: latest_version.to_string(),
                    error: "Found".to_string(),
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Manifest {
                version: "".to_string(),
                error: format!("Error querying bucket {}", e),
            }),
        ),
    }
}

pub async fn latest_firmware(
    State(instance): State<Minio>,
) -> (StatusCode, HeaderMap, std::string::String) {
    let args = ListObjectsArgs::default();
    let query = instance.list_objects("bin", args).await;
    let re = Regex::new(r"^charizhard\.V(\d+\.\d+)\.bin$").unwrap();
    match query {
        Ok(res) => {
            let mut firmware_files: Vec<String> = res
                .contents
                .iter()
                .filter_map(|object| {
                    re.captures(&object.key).and_then(|caps| {
                        Some(caps.get(1)?.as_str().to_string())
                    })
                })
                .collect();

            eprintln!("{:?}", firmware_files);
            firmware_files.sort();

            if let Some(latest_firmware) = firmware_files.last() {
                return get_file(instance, latest_firmware).await;
            } else {
                (
                    StatusCode::NOT_FOUND,
                    HeaderMap::default(),
                    "No firmware files found.".to_string(),
                )
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            HeaderMap::default(),
            format!("Error querying bucket: {}", e),
        ),
    }
}

#[allow(dead_code)]
pub async fn specific_firmware(
    AxumPath(file_name): AxumPath<String>,
    State(instance): State<Minio>,
) -> (StatusCode, HeaderMap, std::string::String) {
    return get_file(instance, &file_name).await;
}

//curl -X POST http://localhost:8080/firmware/charizhard.V1.3.bin \
//  -T ./firmware.bin \
//  -H "Authorization: Bearer $JWT_TOKEN"
pub async fn post_firmware(
    AxumPath(file_name): AxumPath<String>,
    State(instance): State<Minio>,
    request: Request,
) -> (StatusCode, std::string::String) {
    let executor = instance.executor(Method::POST);
    let body = request.into_body();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap(); //cant fail, usize::max is never reached
    let query = executor
        .bucket_name(FIRMWARE_DIR)
        .object_name(file_name)
        .body(bytes)
        .send_ok()
        .await;
    match query {
        Ok(_) => (
            StatusCode::OK,
            "Firmware successfully uploaded !".to_string(),
        ),
        Err(e) => {
            eprintln!("Upload error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error uploading firmware {}", e),
            )
        }
    }
}

//curl -X DELETE http://localhost:8080/firmware/charizhard.V1.3.bin \
//  -T ./firmware.bin \
//  -H "Authorization: Bearer $JWT_TOKEN"
pub async fn delete_firmware(
    AxumPath(file_name): AxumPath<String>,
    State(instance): State<Minio>,
) -> (StatusCode, std::string::String) {
    let executor = instance.executor(Method::DELETE);
    let query = executor
        .bucket_name(FIRMWARE_DIR)
        .object_name(file_name)
        .send_ok()
        .await;
    match query {
        Ok(_) => (
            StatusCode::OK,
            "Firmware successfully deleted !".to_string(),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error deleting firmware {}", e),
        ),
    }
}

pub async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}
