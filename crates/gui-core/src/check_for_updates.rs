// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Checking for updates.  This trait reduces unnecessary rendering and thus
//! reduces CPU & energy usage.
//!
//! `egui` will only redraw the app when there is some sort of interaction
//! unless the app explicitly requests a redraw (immediately or after some
//! length of time).  So that the app is updated correctly even when the user
//! doesn't interact with it (for example, a database query returns some data to
//! be shown), one might be tempted to request a redraw every 150ms (for
//! example).  The problem with this is that it results in unnecessary CPU &
//! energy usage, which is a real problem on battery powered devices.  This
//! trait reduces this problem by making it possible to know when there is data
//! that is being waited on, and when there is not, thus allowing the programme
//! to request a redraw only when needed.
//!

/// Implementing types can check for updates and indicate whether they're
/// waiting for updates.  For example, a channel may need to be checked for
/// database query responses; if it is still waiting it can inform callers so
/// that the checks can be run again at some point in the future.  This reduces
/// blunt checking every Xms, reducing CPU & energy usage.
pub trait CheckForUpdates {
    /// Check for any updates
    fn check_for_updates(&mut self);

    /// Whether the thing is waiting for updates.
    fn waiting_for_updates(&mut self) -> bool;
}
