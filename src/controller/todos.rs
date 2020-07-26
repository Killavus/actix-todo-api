use crate::database::Pool;
use crate::error::WebError;
use crate::forms::todo::{CreateTodo, UpdateTodo};
use crate::model::Todo;

use crate::web_app::ValidatedJson;
use actix_web::{delete, get, patch, post, web, Result};
use serde_json::json;

#[get("")]
async fn list(pool: web::Data<Pool>) -> Result<web::Json<Vec<Todo>>, WebError> {
    Todo::all(&pool).await.map(|vec| web::Json(vec))
}

#[get("/{todoid}")]
async fn fetch(pool: web::Data<Pool>, id: web::Path<i64>) -> Result<web::Json<Todo>, WebError> {
    Todo::find(id.into_inner(), &pool)
        .await
        .map(|todo| web::Json(todo))
}

#[post("")]
async fn create(
    form: ValidatedJson<CreateTodo>,
    pool: web::Data<Pool>,
) -> Result<web::Json<Todo>, WebError> {
    Todo::create(form.into_inner(), &pool)
        .await
        .map(|todo| web::Json(todo))
}

#[patch("/{todoid}")]
async fn update(
    id: web::Path<i64>,
    form: ValidatedJson<UpdateTodo>,
    pool: web::Data<Pool>,
) -> Result<web::Json<Todo>, WebError> {
    let mut todo = Todo::find(id.into_inner(), &pool).await?;
    todo.update(form.into_inner(), &pool).await?;
    Ok(web::Json(todo))
}

#[delete("/{todoid}")]
async fn delete(
    id: web::Path<i64>,
    pool: web::Data<Pool>,
) -> Result<web::Json<serde_json::Value>, WebError> {
    let todo = Todo::find(id.into_inner(), &pool).await?;
    todo.delete(&pool).await?;

    Ok(web::Json(json!({ "status": "ok" })))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(list)
        .service(create)
        .service(fetch)
        .service(update)
        .service(delete);
}
