//!
//! Timeline-related client code
//!

use crate::client::OpenTimelineWebApiClient;
use anyhow::anyhow;
use open_timeline_core::{HasIdAndName, OpenTimelineId, TimelineEdit};

impl OpenTimelineWebApiClient {
    /// `GET` a [`TimelineEdit`] by its ID
    pub async fn get_timeline_edit(&self, id: OpenTimelineId) -> anyhow::Result<TimelineEdit> {
        let url = format!("{}/timeline/{id}", self.api_url());
        Ok(self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<TimelineEdit>()
            .await?)
    }

    /// `PUT` a [`TimelineEdit`]
    pub async fn put_timeline_edit(&self, timeline: &TimelineEdit) -> anyhow::Result<TimelineEdit> {
        let url = format!("{}/timeline", self.api_url());
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

    /// `PATCH` a [`TimelineEdit`]
    pub async fn patch_timeline_edit(
        &self,
        timeline: &TimelineEdit,
    ) -> anyhow::Result<TimelineEdit> {
        let Some(id) = timeline.id() else {
            return Err(anyhow!("Timeline has no ID"));
        };
        let url = format!("{}/timeline/{id}", self.api_url());
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

    /// `DELETE` a [`TimelineEdit`] by its ID
    pub async fn delete_timeline_edit(&self, id: OpenTimelineId) -> anyhow::Result<TimelineEdit> {
        let url = format!("{}/timeline/{id}", self.api_url());
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
