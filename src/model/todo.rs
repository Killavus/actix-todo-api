use crate::database::Pool;
use crate::error::WebError;

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

    ready(Ok(
      HttpResponse::Ok()
        .content_type("application/json")
        .body(body),
    ))
  }
}

#[derive(Debug, Serialize, FromRow)]
pub struct Todo {
  id: i64,
  content: String,
  completed: bool,
}

impl Todo {
  fn new(id: i64, content: String, completed: bool) -> Self {
    Self {
      id,
      content,
      completed,
    }
  }

  pub async fn find(id: i64, pool: &Pool) -> Result<Self, WebError> {
    sqlx::query_as!(Todo, "SELECT * FROM todos WHERE id = ?", id)
      .fetch_one(&*pool)
      .await
      .map_err(|err| WebError::DatabaseError(err))
  }

  pub async fn create(form: CreateTodo, pool: &Pool) -> Result<Self, WebError> {
    // We are going to fetch last row id, it needs to be performed in the same query.
    let mut conn = pool
      .acquire()
      .await
      .map_err(|err| WebError::DatabaseError(err))?;

    #[cfg(target_feature = "postgres")]
    {
      let id = sqlx::query!(
        "INSERT INTO todos (content, completed) VALUES ($1, false) RETURNING id",
        form.content
      )
      .fetch_one(&mut conn)
      .await
      .map(|row: sqlx::postgres::PgRow| row.get("id"))
      .map_err(|err| WebError::DatabaseError(err))?;
      Ok(Self::new(id, form.content, false))
    }

    #[cfg(not(target_feature = "postgres"))]
    {
      sqlx::query!(
        "INSERT INTO todos (content, completed) VALUES (?, false)",
        form.content
      )
      .execute(&mut conn)
      .await
      .map_err(|err| WebError::DatabaseError(err))?;

      let id: (i64,) = sqlx::query_as("SELECT last_insert_rowid()")
        .fetch_one(&mut conn)
        .await
        .map_err(|err| WebError::DatabaseError(err))?;

      Ok(Self::new(id.0, form.content, false))
    }
  }

  pub async fn update(&mut self, mut form: UpdateTodo, pool: &Pool) -> Result<&mut Self, WebError> {
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

    q.execute(&*pool)
      .await
      .map_err(|err| WebError::DatabaseError(err))?;

    if let Some(content) = new_content {
      self.content = content;
    }

    if let Some(completed) = new_completed {
      self.completed = completed;
    }

    Ok(self)
  }

  pub async fn all(pool: &Pool) -> Result<Vec<Self>, WebError> {
    Ok(
      sqlx::query_as!(Todo, "SELECT * FROM todos")
        .fetch_all(&*pool)
        .await
        .map_err(|err| WebError::DatabaseError(err))?,
    )
  }
}
