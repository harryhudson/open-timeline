// SPDX-License-Identifier: MIT

//!
//! Reduced types
//!

mod entities;
mod entity;
mod timeline;
mod timelines;

pub use entities::*;
pub use entity::*;
pub use timeline::*;
pub use timelines::*;

use crate::{Name, OpenTimelineId};
use std::collections::BTreeSet;

// Defer to HasIdAndName trait?
/// Implementing types are "reduced" types - they hold the ID and name of the
/// full type, but nothing else
pub trait IsReducedType {
    /// Instantiate the reduced type using its ID and name
    fn from_id_and_name(id: OpenTimelineId, name: Name) -> Self;

    /// Get the name of the thing
    fn name(&self) -> &Name;

    /// Get the ID of the thing
    fn id(&self) -> OpenTimelineId;
}

// TODO: Is this a good idea? Or should use use as_mut(), as_ref()?
//
// `<<Self as crate::crud::common::IsReducedCollection>::Item>` ensures that
// Item here doesn't conflict with Item in the IntoIterator
//
/// Implementing types are collections of "reduced" types
pub trait IsReducedCollection:
    FromIterator<<Self as IsReducedCollection>::Item> + IntoIterator
{
    type Item: IsReducedType + Ord + Clone;

    fn collection(&self) -> &BTreeSet<<Self as IsReducedCollection>::Item>;
    fn collection_mut(&mut self) -> &mut BTreeSet<<Self as IsReducedCollection>::Item>;
}
