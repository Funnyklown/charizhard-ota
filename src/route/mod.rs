use std::ops::Deref;

use async_std::{
    fs::{self, File},
    path::Path,
    stream::{self, StreamExt},
};
use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use http_body_util::{BodyStream, StreamBody};
use serde_json::{json, Value};
use tokio_util::io::ReaderStream;
use utils::get_file;
mod utils;

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Welcome to Charizhard OTA ! Check /latest/ to get latest firmware"
}

pub async fn latest_firmware() -> (StatusCode, HeaderMap, std::string::String) {
    let firmware_dir = "./bin";

    let entries = match fs::read_dir(firmware_dir).await {
        Ok(entries) => entries,
        Err(_) => {
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
        let file_path = Path::new(firmware_dir).join(latest_firmware);

        let file = match tokio::fs::File::open(&file_path).await {
            Ok(file) => file,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    HeaderMap::default(),
                    "Failed to open firmware file".to_string(),
                );
            }
        };

        get_file(file, latest_firmware).await;
    }

    // If no firmware files are found
    (
        StatusCode::NOT_FOUND,
        HeaderMap::default(),
        "No firmware files found".to_string(),
    )
}
pub async fn specific_firmware() {
    todo!("returns a specific firmware for a given file_name arguments")
}

pub async fn post_firmware() {
    todo!("post firmware to ./bin")
}

async fn delete_firmware() {
    todo!("delete firmware from ./bin")
}
