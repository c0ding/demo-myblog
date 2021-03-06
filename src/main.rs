#[macro_use]
extern crate bson;
#[macro_use]
extern crate anyhow;

use log::info;
use lazy_static::lazy_static;
use mongodb::{Client, Collection};
use actix_web::{web, App, HttpServer, FromRequest};
use crate::article::Article;
use crate::common::*;


mod common;
mod article;

lazy_static! {
    pub static ref MONGO: Client = create_mongo_client();
}

fn create_mongo_client() -> Client {
    Client::with_uri_str("mongodb://localhost:27017")
        .expect("Failed to initialize standalone client.")
}

fn collection(coll_name: &str) -> Collection {
    MONGO.database("myblog").collection(coll_name)
}

fn init_logger() {
    use chrono::Local;
    use std::io::Write;

    let env = env_logger::Env::default()
        .filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    // 设置日志打印格式
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                buf.default_styled_level(record.level()),
                record.module_path().unwrap_or("<unnamed>"),
                &record.args()
            )
        })
        .init();
    info!("env_logger initialized.");
}

#[actix_rt::main]
async fn main() -> std::io::Result<()>{
    init_logger();

    let binding_address = "0.0.0.0:8000";
    HttpServer::new(|| App::new()
        .app_data(
              web::Json::<Article>::configure(|cfg| {
                  cfg.error_handler(|err, req| {
                      log::error!("json extractor error, path={}, {}", req.uri(), err);
                      BusinessError::ArgumentError.into()
                  })
              })
        )
        .service(
            web::scope("/articles")
                .route("", web::get().to(article::list_article))
                .route("", web::post().to(article::save_article))
                .route("{id}", web::put().to(article::update_article))
                .route("{id}", web::delete().to(article::remove_article))
        ))
        .bind(binding_address)
        .expect(&format!("Can not bind to {}", binding_address) )
        .run()
        .await
}
