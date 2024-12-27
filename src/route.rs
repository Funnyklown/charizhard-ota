use async_std::{
    fs::{self, File},
    path::Path,
    stream::StreamExt,
};
use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Welcome to Charizhard OTA ! Check /latest/ to get latest firmware"
}

pub async fn latest_firmware() -> Result<(StatusCode, axum::Json<Value>), axum::Error> {
    let firmware_dir = "./bin";

    // Read the directory contents
    let entries = match fs::read_dir(firmware_dir).await {
        Ok(entries) => entries,
        Err(_) => {
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to read firmware directory"})),
            ));
        }
    };

    // Collect firmware files
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
                return Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Error reading directory entry: {}", err)})),
                ));
            }
        }
    }

    // Sort firmware files to find the latest version
    firmware_files.sort_by(|a, b| a.cmp(b));

    // Get the latest firmware file
    if let Some(latest_firmware) = firmware_files.last() {
        let file_path = Path::new(firmware_dir).join(latest_firmware);

        // Open the file
        let file = match File::open(&file_path).await {
            Ok(file) => file,
            Err(_) => {
                return Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to open firmware file"})),
                ));
            }
        };
        todo!("Fix Stream of latest firmware to file");
        // Create a stream body for the file
        //let stream = ReaderStream::new(file);
        //let body = StreamBody::new(stream);

        // Set headers
        //let mut headers = HeaderMap::new();
        //headers.insert(
        //    axum::http::header::CONTENT_TYPE,
        //    HeaderValue::from_static("application/octet-stream"),
        //);
        //headers.insert(
        //    axum::http::header::CONTENT_DISPOSITION,
        //    HeaderValue::from_str(&format!("attachment; filename=\"{}\"", latest_firmware)).unwrap(),
        //);
        //
        //return (StatusCode::OK, headers, body).into_response();
    }

    // If no firmware files are found
    Ok((
        StatusCode::NOT_FOUND,
        Json(json!({"error": "No firmware files found"})),
    ))
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
