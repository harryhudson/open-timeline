//!
//! Helpers for integration tests
//!

use log::LevelFilter;
use log::info;
use open_timeline_crud::OpenTimelineDatabase;
use open_timeline_www_api::ApiAccessMode;
use open_timeline_www_api::ApiMode;
use open_timeline_www_api::OpenTimelineWebApi;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode};
use sqlx::{Pool, Sqlite};
use std::time::Duration;
use tokio::{task, time::sleep};

/// Setup logging
pub fn setup_test_logging() -> anyhow::Result<()> {
    let config_log = ConfigBuilder::new().add_filter_allow_str("argue").build();

    // TODO: this will return an error if called more than once per process, such as
    // when testing.  Thus we ignore the errors (at least for now)
    let _ = CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        config_log,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )]);
    Ok(())
}

/// Setup test database (migrate the pool supplied to the test)
pub async fn setup_test_database(pool: &Pool<Sqlite>) -> anyhow::Result<()> {
    Ok(OpenTimelineDatabase::migrate_pool(pool).await?)
}

/// Serve API and return the port number
pub async fn setup_test_api_server(pool: &Pool<Sqlite>) -> anyhow::Result<u16> {
    let pool = pool.clone();

    // Get a free port
    let port = portpicker::pick_unused_port().expect("No ports free");

    // Start server
    info!("About to spawn");
    task::spawn(async move {
        info!("About to serve API from new tokio task");
        OpenTimelineWebApi::serve_v1(pool, port, ApiAccessMode::Read, ApiMode::Static)
            .await
            .unwrap();
    });

    // Give the server some time to startup
    sleep(Duration::from_secs_f32(0.5)).await;

    Ok(port)
}
