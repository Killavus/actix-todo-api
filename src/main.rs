#[macro_use]
extern crate validator_derive;

use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use dotenv::dotenv;
use std::env;

mod controller;
mod database;
mod error;
mod forms;
mod model;
mod web_app;

#[actix_rt::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let bind_host = env::var("BIND_HOST").unwrap_or("127.0.0.1:8080".to_string());
    let db_cfg = web::block(|| database::PoolConfig::from_env()).await?;
    let db_pool = database::pool(db_cfg).await?;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::scope("/todos").configure(controller::todos::init))
            .service(web::scope("/work_lists").configure(controller::work_lists::init))
            .data(db_pool.clone())
    })
    .bind(bind_host)?
    .run()
    .await
    .map_err(|err| anyhow::Error::new(err))
}
