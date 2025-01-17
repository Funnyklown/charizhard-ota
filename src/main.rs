use anyhow::Error;
use axum::{
    routing::{get, post},
    Router,
};
use axum_keycloak_auth::{
    instance::{KeycloakAuthInstance, KeycloakConfig},
    layer::KeycloakAuthLayer,
    PassthroughMode,
};
use charizhard_ota::route::{root, specific_firmware};
use reqwest::Url;
use route::{delete_firmware, handle_manifest, latest_firmware, post_firmware};
use std::result::Result::Ok;
mod route;

pub fn public_router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/latest", get(latest_firmware))
        .route("/firmware/:file_name", get(specific_firmware))
        .route("/manifest", get(handle_manifest))
}

pub fn protected_router(instance: KeycloakAuthInstance) -> Router {
    Router::new()
        .route(
            "/firmware/:file_name",
            post(post_firmware).delete(delete_firmware),
        )
        .layer(
            KeycloakAuthLayer::<String>::builder()
                .instance(instance)
                .passthrough_mode(PassthroughMode::Block)
                .persist_raw_claims(false)
                .expected_audiences(vec![String::from("account")])
                .required_roles(vec![String::from("Admin")])
                .build(),
        )
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let keycloak_auth_instance = KeycloakAuthInstance::new(
        KeycloakConfig::builder()
            .server(Url::parse("http://localhost:8080/").unwrap())
            .realm(String::from("charizhard-ota"))
            .build(),
    );

    let router = public_router().merge(protected_router(keycloak_auth_instance));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await?;
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
