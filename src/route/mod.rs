use std::ops::Deref;

use async_std::{
    fs::{self, File},
    path::Path,
    stream::{self, StreamExt},
};
use axum::{http::{HeaderMap, HeaderValue, StatusCode}, response::IntoResponse};
use serde_json::{json, Value};
use http_body_util::{BodyStream, StreamBody};
use tokio_util::io::ReaderStream;
mod utils;

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Welcome to Charizhard OTA ! Check /latest/ to get latest firmware"
}

pub async fn latest_firmware() -> impl IntoResponse {
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
        // Create a stream body for the file
        let mut stream = ReaderStream::new(file);
        let mut body = Vec::new();
        while let Some(chunck) = stream.next().await {
            body.extend_from_slice(&chunck.unwrap());
        }

        let full_body=String::from_utf8(body).unwrap();
        // Set headers
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
        headers.insert(
            axum::http::header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", latest_firmware)).unwrap(),
        );
        
        return (
            StatusCode::OK,
            headers,
            full_body,
        )
    }

    // If no firmware files are found
    (
        StatusCode::NOT_FOUND,
        HeaderMap::default(),
        "No firmware files found".to_string(),
    )
}
async fn specific_firmware() {
    todo!("returns a specific firmware for a given file_name arguments")
}

async fn post_firmware() {
    todo!("post firmware to ./bin")
}

async fn delete_firmware() {
    todo!("delete firmware from ./bin")
}
