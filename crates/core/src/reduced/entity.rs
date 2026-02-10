// SPDX-License-Identifier: MIT

//!
//! Reduced entity
//!

use crate::{IsReducedType, Name, OpenTimelineId};
use serde::{Deserialize, Serialize};

/// The reduced entity type - holds only the [`OpenTimelineId`] and [`Name`] of the
/// full type
#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
pub struct ReducedEntity {
    id: OpenTimelineId,
    name: Name,
}

impl IsReducedType for ReducedEntity {
    fn from_id_and_name(id: OpenTimelineId, name: Name) -> Self {
        Self { id, name }
    }

    fn name(&self) -> &Name {
        &self.name
    }

    fn id(&self) -> OpenTimelineId {
        self.id
    }
}
