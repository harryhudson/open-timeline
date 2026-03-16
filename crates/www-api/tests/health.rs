//!
//! Test the `/health` endpoint
//!

mod common;

use crate::common::*;
use open_timeline_www_api::OpenTimelineWebApiClient;
use reqwest::StatusCode;
use sqlx::{Pool, Sqlite};

#[sqlx::test]
async fn health(pool: Pool<Sqlite>) -> anyhow::Result<()> {
    setup_test_logging()?;
    setup_test_database(&pool).await?;
    let port = setup_test_api_server(&pool).await?;

    // Create client
    let secure_connection = false;
    let domain = String::from("localhost");
    let api_version = 1;
    let api_client =
        OpenTimelineWebApiClient::new(secure_connection, domain, api_version, Some(port));

    // Hit the /health endpoint
    let status = api_client.health_check().await?;
    assert_eq!(status, StatusCode::OK);
    Ok(())
}
