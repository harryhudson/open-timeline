// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Those things used across the OpenTimeline GUI crate
//!

/// Implementing types can be reloaded.  This is used when data changes
/// elsewhere (e.g. when an entity is deleted)
pub trait Reload {
    /// Request that the data held by some data structure is reloaded from the
    /// database.
    fn request_reload(&mut self);

    /// If a reload has been requested one can use this to check for responses
    /// from the database.
    fn check_reload_response(&mut self);
}
