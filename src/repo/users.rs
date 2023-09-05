use super::Repo;
//use anyhow::Result;
use super::DateTime;
use serde::Serialize;
use uuid::Uuid;
//use sqlx::Connection;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::StatusCode,
    headers::Authorization,
};
use http::request::Parts;

#[derive(Debug, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for User
where
    Repo: FromRef<S>,
    S: Send + Sync,
{
    // If anything goes wrong or no session is found, redirect to the auth page
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let repo = Repo::from_ref(state);

        let auth = match parts.headers.get(axum::http::header::AUTHORIZATION) {
            Some(v) => match Authorization::bearer(v.to_str().unwrap()){
                Ok(vv) => vv,
                Err(e) => {
                    log::error!("invalid bearer token '{}': {:?}", v.to_str().unwrap(), e);
                    return Err(StatusCode::UNAUTHORIZED)
                },
            },
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        let user = match repo.get_user_from_session(auth.token().to_string().replace("Bearer ", "")).await {
            Ok(v) => v,
            Err(e) => {
                match e {
                    sqlx::Error::RowNotFound => (),
                    _ => log::error!("error retrieving the user session: {}", e),
                }

                return Err(StatusCode::UNAUTHORIZED);
            }
        };

        Ok(user)
    }
}

struct IdResponse {
    id: Uuid,
}

pub struct PasswordHashResponse {
    pub id: Uuid,
    pub password_hash: String,
}

impl Repo {
    pub async fn create_user(
        &self,
        username: String,
        password: String,
    ) -> Result<Uuid, sqlx::Error> {
        let res = sqlx::query_as!(
            IdResponse,
            "INSERT INTO users (username, password_hash)
                VALUES ($1, $2)
                RETURNING id;",
            username,
            password,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(res.id)
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<(), sqlx::Error> {
        let mut tx = sqlx::Pool::begin(&self.db).await?;

        sqlx::query!(
            "DELETE FROM sessions
            WHERE user_id = $1;",
            &id,
        )
        .execute(&mut tx)
        .await?;

        sqlx::query!(
            "DELETE FROM todos
            WHERE user_id = $1;",
            &id,
        )
        .execute(&mut tx)
        .await?;

        sqlx::query!(
            "DELETE FROM users
            WHERE id = $1;",
            &id,
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn get_password_hash(
        &self,
        username: String,
    ) -> Result<PasswordHashResponse, sqlx::Error> {
        let res = sqlx::query_as!(
            PasswordHashResponse,
            "SELECT id, password_hash
            FROM users
            WHERE username = $1",
            username,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(res)
    }

    pub async fn get_user_from_session(&self, token: String) -> Result<User, sqlx::Error> {
        let res = sqlx::query!(
            "SELECT u.id, u.username, u.created_at, u.updated_at
            FROM users u
            INNER JOIN sessions s ON u.id = s.user_id
            WHERE s.token = $1
            AND s.valid_until > NOW()
            LIMIT 1;",
            token,
        )
        .fetch_one(&self.db)
        .await?;

        let user = User {
            id: res.id,
            username: res.username,
            created_at: res.created_at.into(),
            updated_at: res.updated_at.into(),
        };

        Ok(user)
    }

    pub async fn create_session(&self, user_id: &Uuid, token: &String) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO sessions (user_id, token)
            VALUES ($1, $2);",
            user_id,
            token,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn expire_session(&self, token: &String) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE sessions 
            SET valid_until = NOW()
            WHERE token = $1",
            token,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
