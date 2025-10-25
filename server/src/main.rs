use salvo::prelude::*;
use anyhow::{Context };

pub mod apis;
pub mod core;

#[tokio::main]
async fn main() {
    // env
    dotenvy::dotenv().ok();
    core::app::init_log().unwrap();

    let app_state = core::app::init_app().await.context("init app failed").unwrap();
    let host = app_state.config.server.host.clone();
    let port = app_state.config.server.port;
    let app_service = core::router::create_router(app_state);

    // 启动服务器
    let addr=format!("{}:{}",host,port);
    tracing::info!("Server starting on {}", addr);
    let acceptor = TcpListener::new(addr).bind().await;
    Server::new(acceptor).serve(app_service).await;
}

