mod users;
mod todos;
use crate::repo::Repo;

use axum::{
    extract::{Host, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, any, delete},
    Json, Router,
};

pub async fn router() -> Router<(), hyper::Body> {
    let repo = Repo::new().await.unwrap();

    Router::new()
        .route("/", get(default_page))
        .route("/health", get(health))
        .route(
            "/api/user",
            post(users::create)
                .get(users::get_own_details)
                .delete(users::delete_account),
        )
        .route("/auth/login", post(users::login))
        .route("/auth/logout", any(users::logout))
        .route("/api/todo", get(todos::list).post(todos::create))
        .route("/api/todo/:id", delete(todos::delete))
        .with_state(repo)
}

async fn default_page() -> impl IntoResponse {
    let resp = StatusBody::new("default".to_string());

    (StatusCode::OK, Json(resp))
}

async fn health(
    State(repo): State<Repo>,
    Host(host): Host,
) -> Result<Json<HealthBody>, StatusCode> {
    log::debug!("health check from: '{}'", host);

    let query_count = match repo.active_queries().await {
        Ok(v) => v,
        Err(e) => {
            log::error!("error retrieving database query count: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let res = HealthBody {
        status: "ok".to_string(),
        query_count,
    };

    Ok(Json(res))
}

#[derive(serde::Serialize)]
struct CreateResponse {
    id: uuid::Uuid,
}

#[derive(serde::Serialize)]
struct StatusBody {
    status: String,
}

impl StatusBody {
    fn new(status: String) -> Self {
        Self { status }
    }
}

#[derive(serde::Serialize)]
struct HealthBody {
    status: String,
    query_count: i64,
}
