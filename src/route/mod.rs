//todo clean the the useless unwraps by using anyhow-error response
use async_std::{
    fs::{self, File},
    path::Path,
    stream::{self, StreamExt},
};
use axum::extract::Path as AxumPath;
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
};
use futures_util::TryStreamExt;
use http_body_util::{BodyStream, StreamBody};
use serde_json::{json, Value};
use tokio_util::io::ReaderStream;
use utils::{get_file, stream_to_file};
mod utils;

const FIRMWARE_DIR: &str = "./bin";
// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Welcome to Charizhard OTA ! Check /latest/ to get latest firmware"
}

pub async fn latest_firmware() -> (StatusCode, HeaderMap, std::string::String) {
    let entries = match fs::read_dir(FIRMWARE_DIR).await {
        Ok(entries) => entries,
        Err(e) => {
            println!("{}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                HeaderMap::default(),
                "Failed to read firmware directory".to_string(),
            );
        }
    };

    let mut firmware_files = Vec::new();
    tokio::pin!(entries); // Pin the stream for iteration
    while let Some(entry_result) = entries.next().await {
        match entry_result {
            Ok(entry) => {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with("charizhard.V") && file_name.ends_with(".bin") {
                        firmware_files.push(file_name);
                    }
                }
            }
            Err(err) => {
                println!("{}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    HeaderMap::default(),
                    "Error reading directory entry".to_string(),
                );
            }
        }
    }

    firmware_files.sort_by(|a, b| a.cmp(b));

    if let Some(latest_firmware) = firmware_files.last() {
        let file_path = Path::new(FIRMWARE_DIR).join(latest_firmware);

        let file = match tokio::fs::File::open(&file_path).await {
            Ok(file) => file,
            Err(e) => {
                println!("{}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    HeaderMap::default(),
                    "Failed to open firmware file".to_string(),
                );
            }
        };
        return get_file(file, latest_firmware).await;
    }

    // If no firmware files are found
    (
        StatusCode::NOT_FOUND,
        HeaderMap::default(),
        "No firmware files found".to_string(),
    )
}
pub async fn specific_firmware(
    AxumPath(file_name): AxumPath<String>,
) -> (StatusCode, HeaderMap, std::string::String) {
    let entries = match fs::read_dir(FIRMWARE_DIR).await {
        Ok(entries) => entries,
        Err(e) => {
            println!("{}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                HeaderMap::default(),
                "Failed to read firmware directory".to_string(),
            );
        }
    };

    tokio::pin!(entries); // Pin the stream for iteration
    while let Some(entry_result) = entries.next().await {
        match entry_result {
            Ok(entry) => {
                let caca = entry.file_name().into_string();
                if caca == Ok(file_name.to_string()) {
                    let file = match tokio::fs::File::open(
                        Path::new(FIRMWARE_DIR).join(file_name.clone()),
                    )
                    .await
                    {
                        Ok(file) => file,
                        Err(e) => {
                            println!("{}", e);
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                HeaderMap::default(),
                                "Failed to open firmware file".to_string(),
                            );
                        }
                    };
                    return get_file(file, &file_name).await;
                }
            }
            Err(err) => {
                println!("{}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    HeaderMap::default(),
                    "Error reading directory entry".to_string(),
                );
            }
        }
    }
    /* `(StatusCode, HeaderMap, std::string::String)` value */
    // If no firmware files are found
    (
        StatusCode::NOT_FOUND,
        HeaderMap::default(),
        "Firmware file not found".to_string(),
    )
}
pub async fn post_firmware(
    AxumPath(file_name): AxumPath<String>,
    request: Request,
) -> Result<(), (StatusCode, String)> {
    stream_to_file(&file_name, request.into_body().into_data_stream()).await
}
pub async fn delete_firmware(AxumPath(file_name): AxumPath<String>) -> (StatusCode, HeaderMap, std::string::String) {
    
    let result = tokio::fs::remove_file(Path::new(FIRMWARE_DIR).join(file_name)).await;

    match result {
        Ok(_) => {
            (
                StatusCode::OK,
                HeaderMap::default(),
                "Firmware successfully deleted !".to_string()
            )
        },
        Err(err) => {
            println!("{}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                HeaderMap::default(),
                "Skill issue".to_string()
            )

        }
    }

}
