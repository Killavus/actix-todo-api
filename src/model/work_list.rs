use serde::Serialize;
use sqlx::{self, FromRow};

use super::Todo;
use crate::database::Pool;
use crate::error::WebError;

#[derive(Serialize, Debug, FromRow)]
pub struct WorkList {
    id: i64,
    name: String,
    todos: Vec<Todo>,
}

impl WorkList {
    fn new(id: i64, name: String, todos: Vec<Todo>) -> Self {
        Self { id, name, todos }
    }

    pub async fn find(id: i64, pool: &Pool) -> Result<Self, WebError> {
        let mut conn = pool.acquire().await?;

        let todos = sqlx::query_as!(Todo, "SELECT * FROM todos WHERE todos.work_list_id = ?", id)
            .fetch_all(&mut conn)
            .await?;

        let mut query: Option<(i64, String)> =
            sqlx::query_as("SELECT id, name FROM work_lists WHERE work_lists.id = ?")
                .bind(id)
                .fetch_optional(&mut conn)
                .await?;

        if let Some(row) = query.take() {
            Ok(WorkList::new(row.0, row.1, todos))
        } else {
            Err(WebError::DatabaseError(sqlx::Error::RowNotFound))
        }
    }
}
