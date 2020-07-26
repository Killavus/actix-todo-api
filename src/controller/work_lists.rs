use crate::database::Pool;
use crate::error::WebError;
use crate::forms::work_list::CreateWorkList;
use crate::model::WorkList;
use crate::web_app::{Client, ValidatedJson};

use actix_web::{get, post, web, Result};

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

#[get("")]
async fn list(client: Client, pool: web::Data<Pool>) -> Result<web::Json<Vec<WorkList>>, WebError> {
    WorkList::list(&client, &pool)
        .await
        .map(|collection| web::Json(collection))
}

pub fn init(config: &mut web::ServiceConfig) {
    config.service(fetch).service(list).service(create);
}
