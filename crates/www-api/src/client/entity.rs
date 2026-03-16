//!
//! Entity-related client code
//!

use crate::client::OpenTimelineWebApiClient;
use anyhow::anyhow;
use open_timeline_core::{HasIdAndName, OpenTimelineId, TimelineEdit};

impl OpenTimelineWebApiClient {
    /// `GET` an [`Entity`] by its ID
    pub async fn get_entity(&self, id: OpenTimelineId) -> anyhow::Result<TimelineEdit> {
        let url = format!("{}/entity{id}", self.api_url());
        Ok(self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<TimelineEdit>()
            .await?)
    }

    /// `PUT` an [`Entity`]
    pub async fn put_entity(&self, timeline: &TimelineEdit) -> anyhow::Result<TimelineEdit> {
        let url = format!("{}/entity", self.api_url());
        Ok(self
            .client
            .put(url)
            .json(timeline)
            .send()
            .await?
            .error_for_status()?
            .json::<TimelineEdit>()
            .await?)
    }

    /// `PATCH` an [`Entity`]
    pub async fn patch_entity(&self, timeline: &TimelineEdit) -> anyhow::Result<TimelineEdit> {
        let Some(id) = timeline.id() else {
            return Err(anyhow!("Timeline has no ID"));
        };
        let url = format!("{}/entity{id}", self.api_url());
        Ok(self
            .client
            .patch(url)
            .json(timeline)
            .send()
            .await?
            .error_for_status()?
            .json::<TimelineEdit>()
            .await?)
    }

    /// `DELETE` an [`Entity`] by its ID
    pub async fn delete_entity(&self, id: OpenTimelineId) -> anyhow::Result<TimelineEdit> {
        let url = format!("{}/entity{id}", self.api_url());
        Ok(self
            .client
            .delete(url)
            .send()
            .await?
            .error_for_status()?
            .json::<TimelineEdit>()
            .await?)
    }
}
