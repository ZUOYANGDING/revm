#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use std::sync::Arc;

use clap::Parser;

use config::CLIArguments;
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
    start(cli_args).await
}

/// Start the service
async fn start(cli_args: CLIArguments) -> anyhow::Result<()> {
    // Load configuration
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
    // In that way, need to store with timestamp or change Insert to InsertOrUpdate
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
    let (_, server) =
        warp_serve.bind_with_graceful_shutdown(([0, 0, 0, 0], config.port), async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Irrecoverable error: failed to listen to shutdown signal");
        });
    server.await;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use reqwest;
    use tokio::time::{sleep, Duration};

    /// Integration Test
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn integration_test() {
        let mut cur_dir = std::env::current_dir().unwrap();
        cur_dir.push("example_config_file/config.toml");
        let config_path = cur_dir.to_str().unwrap().to_string();

        // Set the command-line arguments
        let cli_args = CLIArguments { config_path };

        // Start the main function in a separate thread
        let _ = tokio::task::spawn(async move {
            if let Err(e) = start(cli_args).await {
                eprintln!("Failed to start server: {:?}", e);
            }
        });
        // Give the server some time to start
        sleep(Duration::from_secs(10)).await;

        // Send the HTTP request
        let client = reqwest::Client::new();
        let res = client
            .get("http://localhost:8080/token-info?token=WETH")
            .send()
            .await
            .expect("Failed to send request");

        // Check the response
        assert!(res.status().is_success());
        let body = res.text().await.expect("Failed to read response body");
        println!("Response: {}", body);
    }
}
