#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use std::sync::Arc;

use clap::Parser;

use service::fetch_and_store;
use warp::Filter;

mod config;
mod db;
mod error;
mod router;
mod service;
mod sql;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let cli_args = config::CLIArguments::parse();
    let config = config::load_config(&cli_args.config_path)
        .expect("Irrecoveralbe error: failed to load config.toml");

    // Setup Logging (Only support print to terminal)
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level_for("warp", log::LevelFilter::Error)
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;

    // init DB
    let db =
        db::Database::open(config.clone()).expect("Irrecoverable error: failed to open database");
    let db = Arc::new(db);
    // TODO can set up the query and store as a cron job
    match fetch_and_store(&config.swap_pools, db.clone()) {
        Ok(_) => log::info!("Token Data Stored"),
        Err(err) => log::error!("{}", err),
    }

    // start the server
    let warp_serve = warp::serve(
        router::index_route()
            .or(router::get_routers(db.clone()))
            .recover(error::handle_rejection),
    );
    let (_, server) = warp_serve.bind_with_graceful_shutdown(([0, 0, 0, 0], 8080), async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Irrecoverable error: failed to listen to shutdown signal");
    });
    server.await;
    Ok(())
}
