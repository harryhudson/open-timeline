// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! CRUD trait implementations for [`ReducedEntity`]
//!

use crate::{CrudError, FetchById, FetchByName, entity_id_from_name, entity_name_from_id};
use open_timeline_core::{IsReducedType, Name, OpenTimelineId, ReducedEntity};
use sqlx::{Sqlite, Transaction};

impl FetchByName for ReducedEntity {
    async fn fetch_by_name(
        transaction: &mut Transaction<'_, Sqlite>,
        name: &Name,
    ) -> Result<Self, CrudError> {
        let id = entity_id_from_name(transaction, name).await?;
        Ok(ReducedEntity::from_id_and_name(id, name.clone()))
    }
}

impl FetchById for ReducedEntity {
    async fn fetch_by_id(
        transaction: &mut Transaction<'_, Sqlite>,
        id: &OpenTimelineId,
    ) -> Result<Self, CrudError> {
        let name = entity_name_from_id(transaction, id).await?;
        Ok(ReducedEntity::from_id_and_name(*id, name))
    }
}
