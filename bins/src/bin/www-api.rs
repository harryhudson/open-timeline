// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The OpenTimeline www API
//!

use clap::{CommandFactory, Parser};
use open_timeline_www_api::{ApiAccessMode, ApiMode, prepare_api_router};
use std::path::PathBuf;

/// OpenTimeline www API entry point (serve the www JSON API)
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Check the options
    match (&args.database, &args.read_only, &args.dynamic) {
        //----------------------------------------------------------------------
        // Invalid
        // TODO: update the read_only part
        //----------------------------------------------------------------------
        (database, Some(read_only), Some(dynamic)) => {
            let db_url = format!("sqlite://{}", database.to_string_lossy());
            serve(&db_url, *read_only, *dynamic).await
        }
        //----------------------------------------------------------------------
        // Invalid
        //----------------------------------------------------------------------
        _ => {
            eprintln!("CLI Error: invalid options");
            Cli::command().print_long_help().unwrap();
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Serve the website and API
async fn serve(db_url: &str, read_only: bool, dynamic: bool) {
    // Setup up the API modes
    let access_mode = if read_only {
        ApiAccessMode::Read
    } else {
        ApiAccessMode::ReadWrite
    };
    let api_mode = if dynamic {
        ApiMode::Dynamic
    } else {
        ApiMode::Static
    };

    // Get the router
    let api_router = prepare_api_router(db_url, access_mode, api_mode)
        .await
        .unwrap();

    // Specify the IP addr and port number
    let addr = "0.0.0.0:2408";

    // Bind the listener for new connections
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // Print the address
    println!("http://{addr}");

    // Serve the server
    axum::serve(listener, api_router).await.unwrap();
}

/// OpenTimeline CLI args using [clap]
#[derive(Parser, Debug)]
#[command(
    version,
    about = "OpenTimeline www API server",
    after_help = "This is intended for use when deploying to a server and in CI"
)]
pub struct Cli {
    /// Path to the database
    #[arg(long)]
    pub database: PathBuf,

    /// Whether the database should be read-only
    ///
    /// Although this is marked as optional, it is actually required.  It
    /// is optional only so that the usage is `--read-only=<true/false>`
    /// rather than `--read-only`
    #[arg(long)]
    pub read_only: Option<bool>,

    /// Whether the API should be static or dynamic (e.g. should query
    /// parameters be used?)
    ///
    /// Although this is marked as optional, it is actually required.  It
    /// is optional only so that the usage is `--dynamic=<true/false>`
    /// rather than `--dynamic`
    #[arg(long)]
    pub dynamic: Option<bool>,
}
