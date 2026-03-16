//!
//! Helpers for client use of the web API
//!

mod entity;
mod timeline;

use reqwest::{Client, StatusCode};

/// Helper for using the argue JSON web API
#[derive(Debug)]
pub struct OpenTimelineWebApiClient {
    /// The connection client
    client: reqwest::Client,

    /// Whether to connect securely
    secure_connection: bool,

    /// e.g. "localhost", "www.argue.com"
    domain: String,

    /// The API version number
    api_version: u8,

    /// The port to use, if one should be used (~65,000 limit is because of TCP,
    /// not file descriptor limits)
    port: Option<u16>,
}

impl OpenTimelineWebApiClient {
    /// Get a new client
    pub fn new(
        secure_connection: bool,
        domain: String,
        api_version: u8,
        port: Option<u16>,
    ) -> Self {
        Self {
            client: Client::new(),
            secure_connection,
            domain,
            api_version,
            port,
        }
    }

    /// Get the URL for the site (e.g. "http://localhost:5050")
    pub fn site_url(&self) -> String {
        let protocol = if self.secure_connection {
            "https"
        } else {
            "http"
        };
        let domain = &self.domain;
        let port = match self.port {
            Some(port) => format!(":{port}"),
            None => "".to_string(),
        };
        format!("{protocol}://{domain}{port}")
    }

    /// Get the URL for the API version base (e.g. "http://localhost:5050/api/v1")
    pub fn api_url(&self) -> String {
        let site_url = self.site_url();
        let api_version = self.api_version.to_string();
        format!("{site_url}/api/v{api_version}")
    }

    /// Check API server is up & running (`/health` check)
    pub async fn health_check(&self) -> anyhow::Result<StatusCode> {
        let url = format!("{}/health", self.site_url());
        Ok(self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .status())
    }
}
