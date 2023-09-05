mod repo;
mod services;
use anyhow::Result;
use std::net::SocketAddr;
use hyper::Server;

#[cfg(target_arch = "wasm32")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    run_main().await
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    run_main().await
}

async fn run_main() -> Result<()> {
    pretty_env_logger::init();

    log::info!("starting server...");
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let router = crate::services::router().await;
    let server = Server::bind(&addr).serve(router.into_make_service());

    log::info!("server running on {}", addr);
    if let Err(err) = server.await {
        log::error!("error while serving: {}", err);
    }

    Ok(())
}
