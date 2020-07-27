use actix_web::{delete, patch, post, web, Result};
use serde_json::json;

use crate::database::Pool;
use crate::error::WebError;
use crate::forms::todo::{CreateTodo, UpdateTodo};
use crate::model::Todo;

use crate::web_app::{Client, ValidatedJson};

#[post("")]
async fn create(
    form: ValidatedJson<CreateTodo>,
    client: Client,
    pool: web::Data<Pool>,
) -> Result<web::Json<Todo>, WebError> {
    Todo::create(form.into_inner(), &client, &pool)
        .await
        .map(|todo| web::Json(todo))
}

#[patch("/{todoid}")]
async fn update(
    id: web::Path<i64>,
    form: ValidatedJson<UpdateTodo>,
    client: Client,
    pool: web::Data<Pool>,
) -> Result<web::Json<Todo>, WebError> {
    let mut todo = Todo::find(id.into_inner(), &client, &pool).await?;
    todo.update(form.into_inner(), &client, &pool).await?;
    Ok(web::Json(todo))
}

#[delete("/{todoid}")]
async fn delete(
    id: web::Path<i64>,
    client: Client,
    pool: web::Data<Pool>,
) -> Result<web::Json<serde_json::Value>, WebError> {
    let todo = Todo::find(id.into_inner(), &client, &pool).await?;
    todo.delete(&client, &pool).await?;

    Ok(web::Json(json!({ "status": "ok" })))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(create).service(update).service(delete);
}
