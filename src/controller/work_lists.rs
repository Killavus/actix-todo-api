use actix_web::{delete, get, patch, post, web, Result};
use serde_json::json;

use crate::database::Pool;
use crate::error::WebError;
use crate::forms::work_list::{CreateWorkList, UpdateWorkList};
use crate::model::WorkList;
use crate::web_app::{Client, ValidatedJson};

#[get("{id}")]
async fn fetch(
    id: web::Path<i64>,
    client: Client,
    pool: web::Data<Pool>,
) -> Result<web::Json<WorkList>, WebError> {
    let id = id.into_inner();

    WorkList::find(id, &client, &pool)
        .await
        .map(|work_list| web::Json(work_list))
}

#[post("")]
async fn create(
    client: Client,
    form: ValidatedJson<CreateWorkList>,
    pool: web::Data<Pool>,
) -> Result<web::Json<WorkList>, WebError> {
    WorkList::create(form.into_inner(), &client, &pool)
        .await
        .map(|work_list| web::Json(work_list))
}

#[delete("{id}")]
async fn delete(
    id: web::Path<i64>,
    client: Client,
    pool: web::Data<Pool>,
) -> Result<web::Json<serde_json::Value>, WebError> {
    let work_list = WorkList::find(id.into_inner(), &client, &pool).await?;
    work_list.delete(&client, &pool).await?;
    Ok(web::Json(json!({ "status": "ok" })))
}

#[patch("{id}")]
async fn update(
    id: web::Path<i64>,
    form: ValidatedJson<UpdateWorkList>,
    client: Client,
    pool: web::Data<Pool>,
) -> Result<web::Json<WorkList>, WebError> {
    let mut work_list = WorkList::find(id.into_inner(), &client, &pool).await?;
    work_list.update(&client, form.into_inner(), &pool).await?;

    Ok(web::Json(work_list))
}

#[get("")]
async fn list(client: Client, pool: web::Data<Pool>) -> Result<web::Json<Vec<WorkList>>, WebError> {
    WorkList::list(&client, &pool)
        .await
        .map(|collection| web::Json(collection))
}

pub fn init(config: &mut web::ServiceConfig) {
    config
        .service(fetch)
        .service(list)
        .service(create)
        .service(update)
        .service(delete);
}
