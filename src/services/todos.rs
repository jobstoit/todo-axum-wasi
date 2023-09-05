use crate::repo::{Repo, User, todos::Todo};
use super::CreateResponse;
use axum::{
    http::StatusCode,
    extract::{Path, State},
    Json
};
use uuid::Uuid;
use validator::Validate;

#[derive(serde::Deserialize, Validate, Default)]
pub(super) struct CreateTodoRequest {
    #[validate(length(min = 1))]
    title: String,
}

pub(super) async fn create(
    user: User,
    State(repo): State<Repo>,
    Json(payload): Json<CreateTodoRequest>,
) -> Result<Json<CreateResponse>, StatusCode> {
    if let Err(_) = payload.validate() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY)
    }

    let id = match repo.create_todo(&user.id, &payload.title).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("error creating new todo: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    };

    let res = CreateResponse{id};

    Ok(Json(res))
}

pub(super) async fn delete(
    user: User,
    State(repo): State<Repo>,
    Path(id): Path<Uuid>,
) -> Result<(), StatusCode> {
    match repo.delete_todo(&user.id, &id).await {
        Ok(_) => Ok(()),
        Err(e) => match e {
            sqlx::Error::RowNotFound => return Err(StatusCode::NOT_FOUND),
            _ => {
            log::error!("error deleting todo '{}': {:?}", &id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub(super) async fn list(
    user: User,
    State(repo): State<Repo>,
) -> Result<Json<Vec<Todo>>, StatusCode> {
    let list = match repo.list_todo(&user.id).await{
        Ok(v) => v,
        Err(e) => {
            log::error!("unable to list todos: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    };

    Ok(Json(list))
}
