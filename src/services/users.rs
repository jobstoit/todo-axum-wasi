use crate::repo::{Repo, User};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use sha2::Sha256;
use super::CreateResponse;
use std::collections::BTreeMap;
use axum::{
    extract::{Json, State, TypedHeader},
    headers::{Authorization, authorization::Bearer},
    http::StatusCode,
    response::Result,
};
use validator::Validate;

#[derive(serde::Deserialize, Default, Validate)]
pub(super) struct CreateUserRequest {
    #[validate(length(min = 5))]
    username: String,
    #[validate(length(min = 5))]
    password: String,
}

pub(super) async fn create(
    State(repo): State<Repo>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<CreateResponse>, StatusCode> {
    if let Err(_) = payload.validate() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY)
    }

    let hash = match hash_password(&payload.password) {
        Ok(v) => v,
        Err(e) => {
            log::error!("error hashing password '{}': {}", &payload.password, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    };


    let id = match repo.create_user(payload.username, hash.to_string()).await {
        Ok(v) => v,
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                return Err(StatusCode::BAD_REQUEST)
            }
            _ => {
                log::error!("error creating the user: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    };

    let res = CreateResponse{id};

    Ok(Json(res))
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct LoginRequest {
    username: String,
    password: String,
}

#[derive(serde::Serialize)]
pub(super) struct LoginResponse {
    token: String,
}

pub(super) async fn login(
    State(repo): State<Repo>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let details = match repo.get_password_hash(payload.username).await {
        Ok(v) => v,
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                return Err(StatusCode::UNAUTHORIZED)
            },
            _ => {
                log::error!("error getting a password");
                return Err(StatusCode::INTERNAL_SERVER_ERROR)
            },
        },
    };

    let verified = verify_password(&payload.password, &details.password_hash);
    if !verified {
        return Err(StatusCode::UNAUTHORIZED)
    }

    let id = details.id;
    let token = create_jwt(id.to_string());

    if let Err(e) = repo.create_session(&id, &token).await {
        log::error!("error creating new session: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR)
    }

    let res = LoginResponse{token};

    Ok(Json(res))
}

//#[cfg(not(target_arch = "wasm32"))]
fn verify_password(password: &String, hash: &String) -> bool {
    bcrypt::verify(password, hash).unwrap() 
}

//#[cfg(not(target_arch = "wasm32"))]
fn hash_password(password: &String) -> anyhow::Result<String> {
    let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;

    Ok(hash)
}

pub(super) async fn logout(
    State(repo): State<Repo>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
) -> Result<(), StatusCode> {
        let auth = match auth_header {
            Some(v) => v,
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        match repo.expire_session(&auth.token().to_string()).await {
            Ok(_) => Ok(()),
            Err(e) => match e {
                sqlx::Error::RowNotFound => return Err(StatusCode::UNAUTHORIZED),
                _ => {
                    log::error!("error expiring session: {:?}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            },
        }
}

fn create_jwt(id: String) -> String {
    let key: Hmac<Sha256> = Hmac::new_from_slice(b"some-secret").unwrap();    
    let mut claims = BTreeMap::new();
    claims.insert("sub", id);
    claims.insert("time", chrono::Local::now().to_string());
    let token_str = claims.sign_with_key(&key).unwrap();

    token_str
}

pub(super) async fn get_own_details(
    user: User,
) -> Result<Json<User>, StatusCode> {
    Ok(Json(user))
}

pub(super) async fn delete_account(
    user: User,
    State(repo): State<Repo>,
) -> Result<(), StatusCode> {
    if let Err(e) = repo.delete_user(user.id).await {
        log::error!("error deleting user: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR)
    }

    Ok(())
}
