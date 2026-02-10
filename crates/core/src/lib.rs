// SPDX-License-Identifier: MIT

//!
//! *Part of the wider OpenTimeline project*
//!
//! This crate defines the basic datatypes used across the OpenTimeline project
//! (web API, desktop application, renderer).
//!
//! This crate is designed to be used by the rest of the OpenTimeline project,
//! as well as by other 3rd party projects that want to interact with
//! OpenTimeline (e.g. via it's JSON web API).
//!
//! This crate aims to provide APIs for each type so that if a type is
//! instantiated, the developer can be sure it's valid.
//!

mod date;
mod entity;
mod id;
mod name;
mod reduced;
mod timeline_edit;
mod timeline_view;

pub use date::*;
pub use entity::*;
pub use id::*;
pub use name::*;
pub use reduced::*;
pub use timeline_edit::*;
pub use timeline_view::*;

// TODO: is this used anywhere (variants could/should hold the more specific Errors)
/// Errors that can be returned by OpenTimeline
#[derive(Debug)]
pub enum OpenTimelineError {
    Date,
    Entity,
    Id,
    Name,
    Tags,
    Timeline,
}

/// Mark that a type has both an [`OpenTimelineId`] and a [`Name`], and setup
/// getters and setters for both
pub trait HasIdAndName {
    /// Get the ID
    fn id(&self) -> Option<OpenTimelineId>;

    /// Set the ID - the [`OpenTimelineId`] passed in must have been
    /// initialised, and therefore is guaranteed to be valid
    fn set_id(&mut self, id: OpenTimelineId);

    /// Get the name
    fn name(&self) -> &Name;

    /// Set the name - the [`Name`] passed in must have been initialised, and
    /// therefore is guaranteed to be valid
    fn set_name(&mut self, name: Name);
}
