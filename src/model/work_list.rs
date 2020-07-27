use serde::Serialize;
use sqlx::{self, FromRow};
use std::collections::HashMap;

use super::Todo;
use crate::database::Pool;
use crate::error::WebError;
use crate::forms::work_list::{CreateWorkList, UpdateWorkList};
use crate::web_app::Client;

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

    pub async fn create(
        form: CreateWorkList,
        client: &Client,
        pool: &Pool,
    ) -> Result<Self, WebError> {
        let mut conn = pool.acquire().await?;

        #[cfg(target_feature = "postgres")]
        {
            let name = form.name.clone();
            let client_id = client.id();
            let id = sqlx::query!(
                "INSERT INTO work_lists (name, client_id) VALUES ($1, false, $2) RETURNING id",
                name,
                client_id
            )
            .fetch_one(&mut conn)
            .await
            .map(|row: database::Row| row.get("id"))?;

            Ok(Self::new(id, form.name, vec![]))
        }

        #[cfg(not(target_feature = "postgres"))]
        {
            let name = form.name.clone();
            let client_id = client.id();
            sqlx::query!(
                "INSERT INTO work_lists (name, client_id) VALUES (?, ?)",
                name,
                client_id
            )
            .execute(&mut conn)
            .await?;

            let id: (i64,) = sqlx::query_as("SELECT last_insert_rowid()")
                .fetch_one(&mut conn)
                .await?;

            Ok(Self::new(id.0, form.name, vec![]))
        }
    }

    pub async fn list(client: &Client, pool: &Pool) -> Result<Vec<Self>, WebError> {
        let mut conn = pool.acquire().await?;

        let work_lists_query: Vec<(i64, String)> =
            sqlx::query_as("SELECT id, name FROM work_lists WHERE work_lists.client_id = ?")
                .bind(client.id())
                .fetch_all(&mut conn)
                .await?;

        // FIXME: There is no way to bind IN-list parameter in sqlx reliably now - let's create it manually.
        let todos_sql = format!(
            "SELECT * FROM todos WHERE todos.work_list_id IN ({})",
            work_lists_query
                .iter()
                .map(|wl| wl.0.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let todos: Vec<Todo> = sqlx::query_as(&todos_sql).fetch_all(&mut conn).await?;
        let mut todos_map = todos.into_iter().fold(HashMap::new(), |mut map, todo| {
            map.entry(todo.work_list_id).or_insert(vec![]).push(todo);
            map
        });

        Ok(work_lists_query
            .into_iter()
            .map(|(id, name)| Self::new(id, name, todos_map.remove(&id).unwrap_or(vec![])))
            .collect())
    }

    pub async fn delete(self, client: &Client, pool: &Pool) -> Result<(), WebError> {
        let client_id = client.id();
        let rows_affected: u64 = sqlx::query!(
            "DELETE FROM work_lists WHERE id = ? AND client_id = ?",
            self.id,
            client_id
        )
        .execute(&*pool)
        .await?;

        if rows_affected > 0 {
            Ok(())
        } else {
            Err(WebError::DatabaseError(sqlx::Error::RowNotFound))
        }
    }

    pub async fn update(
        &mut self,
        client: &Client,
        form: UpdateWorkList,
        pool: &Pool,
    ) -> Result<&mut Self, WebError> {
        let client_id = client.id();
        let new_name = form.name.clone();
        let rows_affected: u64 = sqlx::query!(
            "UPDATE work_lists SET name = ? WHERE id = ? AND client_id = ?",
            new_name,
            self.id,
            client_id
        )
        .execute(&*pool)
        .await?;

        if rows_affected > 0 {
            self.name = form.name;
            Ok(self)
        } else {
            Err(WebError::DatabaseError(sqlx::Error::RowNotFound))
        }
    }

    pub async fn find(id: i64, client: &Client, pool: &Pool) -> Result<Self, WebError> {
        let mut conn = pool.acquire().await?;

        let todos = sqlx::query_as!(Todo, "SELECT * FROM todos WHERE todos.work_list_id = ?", id)
            .fetch_all(&mut conn)
            .await?;

        let mut query: Option<(i64, String)> = sqlx::query_as(
            "SELECT id, name FROM work_lists WHERE work_lists.id = ? AND work_lists.client_id = ?",
        )
        .bind(id)
        .bind(client.id())
        .fetch_optional(&mut conn)
        .await?;

        if let Some(row) = query.take() {
            Ok(WorkList::new(row.0, row.1, todos))
        } else {
            Err(WebError::DatabaseError(sqlx::Error::RowNotFound))
        }
    }
}
