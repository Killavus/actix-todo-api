use crate::database::Pool;
use crate::error::WebError;
use crate::model::WorkList;

use actix_web::{get, web, Result};

#[get("{id}")]
async fn fetch(id: web::Path<i64>, pool: web::Data<Pool>) -> Result<web::Json<WorkList>, WebError> {
    let id = id.into_inner();

    WorkList::find(id, &pool).await.map(|list| web::Json(list))
}

pub fn init(config: &mut web::ServiceConfig) {
    config.service(fetch);
}
