// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! CRUD trait implementations for [`ReducedTimeline`]
//!

use crate::{CrudError, FetchById, FetchByName, timeline_id_from_name, timeline_name_from_id};
use open_timeline_core::{IsReducedType, Name, OpenTimelineId, ReducedTimeline};
use sqlx::{Sqlite, Transaction};

impl FetchByName for ReducedTimeline {
    async fn fetch_by_name(
        transaction: &mut Transaction<'_, Sqlite>,
        name: &Name,
    ) -> Result<Self, CrudError> {
        let id = timeline_id_from_name(transaction, name).await?;
        Ok(ReducedTimeline::from_id_and_name(id, name.clone()))
    }
}

impl FetchById for ReducedTimeline {
    async fn fetch_by_id(
        transaction: &mut Transaction<'_, Sqlite>,
        id: &OpenTimelineId,
    ) -> Result<Self, CrudError> {
        if let Ok(name) = timeline_name_from_id(transaction, id).await {
            Ok(ReducedTimeline::from_id_and_name(*id, name))
        } else {
            Err(CrudError::IdNotInDb)
        }
    }
}
