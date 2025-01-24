use async_std::prelude::Stream;
use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
    BoxError,
};
use futures_util::TryStreamExt;
use minio_rsc::Minio;
use reqwest::Method;
use std::io;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;

use super::FIRMWARE_DIR;

pub async fn get_file(
    instance: Minio,
    file_name: &String,
) -> (StatusCode, HeaderMap, std::string::String) {
    let executor = instance.executor(Method::GET);
    let query = executor
        .bucket_name(FIRMWARE_DIR)
        .object_name(file_name.clone())
        .send_ok()
        .await;
    match query {
        Ok(res) => {
            let body = res.bytes().await;
            match body {
                Ok(bytes) => {
                    let content = String::from_utf8_lossy(&bytes).to_string();
                    let mut headers = HeaderMap::new();
                    headers.insert("Content-Type", "application/octet-stream".parse().unwrap());
                    headers.insert(
                        "Content-Disposition",
                        format!("attachment; filename=\"{}\"", file_name)
                            .parse()
                            .unwrap(),
                    );
                    return (
                        StatusCode::OK,
                        headers,
                        format!("Firmware successfully downloaded: {}", content),
                    );
                }
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        HeaderMap::default(),
                        format!("Failed to read object content: {}", e),
                    )
                }
            }
        }
        Err(e) => return (StatusCode::NOT_FOUND, HeaderMap::default(), e.to_string()),
    }
}

pub async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(path) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }

    async {
        // Convert the stream into an `AsyncRead`.

        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        tokio::pin!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let path = std::path::Path::new("./bin").join(path);
        let mut file = BufWriter::new(File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

// to prevent directory traversal attacks we ensure the path consists of exactly one normal
// component
fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}
