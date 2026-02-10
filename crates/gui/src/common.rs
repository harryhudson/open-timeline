// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Those things used across the OpenTimeline GUI crate
//!

use crate::config::SharedConfig;
use open_timeline_core::{Name, OpenTimelineId};
use open_timeline_crud::{Create, CrudError, DeleteById, FetchByName, Update};
use open_timeline_gui_core::CreateOrEdit;
use std::fmt::Debug;
use tokio::sync::mpsc::Sender;

/// Used to indicate whether the CRUD operation was create/update or delete
#[derive(Debug, Clone)]
pub enum CrudOperationRequested {
    CreateOrUpdate,
    Delete,
}

/// Implementing types can be validated (asynchronously in a new thread against
/// the database if necessary) and converted from their GUI input representation
/// into their database representation.
pub trait ToOpenTimelineType<T> {
    /// Used to get the database representation of the data from GUI
    /// representation now that we know that it is valid.
    fn to_opentimeline_type(&self) -> T;
}

/// A helper function to run Edit or Create CRUD functions which sends the
/// `Result` of the operation down a supplied channel.  This function opens
/// its own database connection and transaction, and commits the transaction
/// after the running of the target CRUD operation if it is successful.
pub async fn save_crud<T>(
    shared_config: SharedConfig,
    edit_or_create: &CreateOrEdit,
    mut value: T,
    tx: Sender<Result<T, CrudError>>,
) where
    T: Create + Update,
{
    let result = async {
        let mut transaction = shared_config.read().await.db_pool.begin().await?;
        match edit_or_create {
            CreateOrEdit::Create => value.create(&mut transaction).await?,
            CreateOrEdit::Edit => value.update(&mut transaction).await?,
        };
        // TODO: is this the correct error variant?
        transaction.commit().await.map_err(|_| CrudError::DbError)?;
        Ok(value)
    }
    .await;
    let _ = tx.send(result).await;
}

// TODO: can we do a similar thing for search by partial name?
pub async fn _fetch_crud<T>(
    shared_config: SharedConfig,
    name: Name,
    tx: Sender<Result<(), CrudError>>,
) where
    T: FetchByName,
{
    let result = async {
        let mut transaction = shared_config.read().await.db_pool.begin().await?;
        T::fetch_by_name(&mut transaction, &name).await?;
        Ok(())
    }
    .await;
    let _ = tx.send(result).await;
}

// TODO: this is almost identical to the above fetch_crud() (and not a million
// miles off the create/update generic)
/// A helper function to run Delete CRUD functions which sends the
/// `Result` of the operation down a supplied channel.  This function opens
/// its own database connection and transaction, and commits the transaction
/// after the running of the target CRUD operation if it is successful.
pub async fn delete_from_id_crud<T>(
    shared_config: SharedConfig,
    id: OpenTimelineId,
    tx: Sender<Result<(), CrudError>>,
) where
    T: DeleteById,
{
    let result = async {
        let mut transaction = shared_config.read().await.db_pool.begin().await?;
        T::delete_by_id(&mut transaction, &id).await?;
        // TODO: is this the correct error variant?
        transaction.commit().await.map_err(|_| CrudError::DbError)?;
        Ok(())
    }
    .await;
    let _ = tx.send(result).await;
}
