use crate::database::Pool;
use crate::error::WebError;
use crate::web_app::Client;

use crate::forms::todo::{CreateTodo, UpdateTodo};

use actix_web::{error::Error, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};
use serde::Serialize;
use sqlx::FromRow;

impl Responder for Todo {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct Todo {
    pub id: i64,
    pub content: String,
    pub completed: bool,
    pub work_list_id: i64,
}

impl Todo {
    async fn authorize(work_list_id: i64, client: &Client, pool: &Pool) -> Result<(), WebError> {
        let result: (bool,) =
            sqlx::query_as("SELECT ? IN (SELECT id FROM work_lists WHERE client_id = ?)")
                .bind(work_list_id)
                .bind(client.id())
                .fetch_one(&*pool)
                .await?;

        if result.0 == false {
            Err(WebError::Unauthorized)
        } else {
            Ok(())
        }
    }

    fn new(id: i64, content: String, completed: bool, work_list_id: i64) -> Self {
        Self {
            id,
            content,
            completed,
            work_list_id,
        }
    }

    pub async fn find(id: i64, client: &Client, pool: &Pool) -> Result<Self, WebError> {
        let client_id = client.id();
        sqlx::query_as!(Todo, "SELECT todos.* FROM todos JOIN work_lists ON work_lists.id = todos.work_list_id WHERE todos.id = ? AND work_lists.client_id = ?", id, client_id)
            .fetch_one(&*pool)
            .await
            .map_err(|err| err.into())
    }

    pub async fn create(form: CreateTodo, client: &Client, pool: &Pool) -> Result<Self, WebError> {
        // We are going to fetch last row id, it needs to be performed in the same query.
        let mut conn = pool.acquire().await?;
        Self::authorize(form.work_list_id, &client, pool).await?;

        #[cfg(target_feature = "postgres")]
        {
            let id = sqlx::query!(
        "INSERT INTO todos (content, completed, work_list_id) VALUES ($1, false, $2) RETURNING id",
        form.content,
        form.work_list_id
      )
            .fetch_one(&mut conn)
            .await
            .map(|row: database::Row| row.get("id"))?;
            Ok(Self::new(id, form.content, false, form.work_list_id))
        }

        #[cfg(not(target_feature = "postgres"))]
        {
            sqlx::query!(
                "INSERT INTO todos (content, completed, work_list_id) VALUES (?, false, ?)",
                form.content,
                form.work_list_id
            )
            .execute(&mut conn)
            .await?;

            let id: (i64,) = sqlx::query_as("SELECT last_insert_rowid()")
                .fetch_one(&mut conn)
                .await?;

            Ok(Self::new(id.0, form.content, false, form.work_list_id))
        }
    }

    pub async fn update(
        &mut self,
        mut form: UpdateTodo,
        client: &Client,
        pool: &Pool,
    ) -> Result<&mut Self, WebError> {
        Self::authorize(self.work_list_id, &client, pool).await?;

        let mut where_list = Vec::with_capacity(2);
        let new_content = form.content.take();
        let new_completed = form.completed.take();

        if new_content.is_some() {
            where_list.push("content = ?".to_owned());
        }

        if new_completed.is_some() {
            where_list.push("completed = ?".to_owned());
        }

        let sql = format!(
            "UPDATE todos SET {} WHERE id = {}",
            where_list.join(", "),
            where_list.len()
        );

        let mut q = sqlx::query(&sql);

        if let Some(content) = new_content.as_ref() {
            q = q.bind(content);
        }

        if let Some(completed) = new_completed.as_ref() {
            q = q.bind(completed);
        }

        q.execute(&*pool).await?;

        if let Some(content) = new_content {
            self.content = content;
        }

        if let Some(completed) = new_completed {
            self.completed = completed;
        }

        Ok(self)
    }

    pub async fn delete(self, client: &Client, pool: &Pool) -> Result<(), WebError> {
        Self::authorize(self.work_list_id, &client, pool).await?;

        sqlx::query!("DELETE FROM todos WHERE id = ?", self.id)
            .execute(&*pool)
            .await?;

        Ok(())
    }
}
