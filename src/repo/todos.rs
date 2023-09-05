use super::{Repo, DateTime};
use uuid::Uuid;

#[derive(serde::Serialize)]
pub struct Todo {
    id: Uuid,
    title: String,
    created_at: DateTime,
    updated_at: DateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    finished_at: Option<DateTime>,
}

impl Repo {
    pub async fn create_todo(&self, user_id: &Uuid, title: &String) -> Result<Uuid, sqlx::Error> {
        let res = sqlx::query!(
            "INSERT INTO todos (user_id, title)
            VALUES ($1, $2)
            RETURNING id;",
            user_id, title,
        ).fetch_one(&self.db).await?;

        Ok(res.id)
    }

    pub async fn delete_todo(&self, user_id: &Uuid, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM todos
            WHERE user_id = $1
            AND id = $2;",
            user_id, id,
        ).execute(&self.db).await?;

        Ok(())
    }

    pub async fn list_todo(&self, user_id: &Uuid) -> Result<Vec<Todo>, sqlx::Error> {
        let list = sqlx::query!(
            "SELECT id, title, created_at, updated_at, finished_at
            FROM todos
            WHERE user_id = $1;",
            user_id,
        ).fetch_all(&self.db).await?
            .into_iter()
            .map(|row| Todo{
                id: row.id,
                title: row.title,
                created_at: row.created_at.into(),
                updated_at: row.updated_at.into(),
                finished_at: match row.finished_at {
                    Some(v) => Some(v.into()),
                    None => None,
                },
            }).collect();

        Ok(list)
    }
}
