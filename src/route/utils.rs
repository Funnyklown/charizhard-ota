use async_std::{prelude::Stream, stream::StreamExt};
use axum::{
    body::Bytes,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    BoxError,
};
use futures_util::TryStreamExt;
use std::io;
use tokio::{fs::File, io::BufWriter, sync::futures};
use tokio_util::io::{ReaderStream, StreamReader};

pub async fn get_file(
    file: File,
    filename: &String,
) -> (StatusCode, HeaderMap, std::string::String) {
    // Create a stream body for the file
    let mut stream = ReaderStream::new(file);
    let mut body = Vec::new();
    while let Some(chunck) = stream.next().await {
        body.extend_from_slice(&chunck.unwrap());
    }

    let full_body = String::from_utf8(body).unwrap();
    // Set headers
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap(),
    );

    return (StatusCode::OK, headers, full_body);
}

async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
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
