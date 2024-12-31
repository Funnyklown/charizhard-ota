//todo implement keycloak logic jwt, token ...
//sudo kc  start-dev --http-port=8180
use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, HeaderMap, StatusCode},
};
use jsonwebtoken::{decode, jwk::JwkSet, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct KeycloakClaims {
    sub: String,
    preferred_username: String,
    email: Option<String>,
    realm_access: Option<RealmAccess>,
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct RealmAccess {
    roles: Vec<String>,
}

static KEYCLOAK_JWKS_URL: &str =
    "http://localhost:8180/realms/charizhard-ota/protocol/openid-connect/certs";

async fn fetch_jwks() -> Result<JwkSet, reqwest::Error> {
    let jwks = reqwest::get(KEYCLOAK_JWKS_URL)
        .await?
        .json::<JwkSet>()
        .await?;
    Ok(jwks)
}


