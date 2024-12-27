use anyhow::Error;
use axum::{routing::get, Router};
use charizhard_ota::route::root;
use route::latest_firmware;
use std::result::Result::Ok;
mod route;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/latest", get(latest_firmware().await.unwrap()));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
